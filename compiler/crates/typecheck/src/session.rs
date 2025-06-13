use crate::{
    constraints::{Constraints, Ty},
    context::FeedbackProvider,
};
use itertools::Itertools;
use petgraph::{
    Direction,
    prelude::{DiGraphMap, UnGraphMap},
    unionfind::UnionFind,
};
use std::{cell::RefCell, collections::BTreeMap, mem};
use wipple_compiler_trace::{AnyRule, NodeId, Rule, rule};

rule! {
    /// The type was unified with another type.
    unified: Extra;
}

#[derive(Clone)]
pub struct Session {
    nodes: Vec<NodeId>,
    constraints: Constraints,
    unify: UnGraphMap<NodeId, AnyRule>,
}

impl Session {
    pub fn from_constraints(
        nodes: impl IntoIterator<Item = NodeId>,
        constraints: Constraints,
    ) -> Self {
        let mut unify = UnGraphMap::<NodeId, AnyRule>::new();
        for (&node, tys) in &constraints.tys {
            for (ty, rule) in tys {
                // We don't need to deeply traverse the type because all types
                // here represent the top-level type of a node. For example, if
                // we have `x :: (1) -> _` and `y :: (1) -> _`, then at this
                // point the (1) must be attached to another specific node in
                // the AST (which will be handled in another iteration), or it
                // can't be inferred anyway.
                if let Ty::Of(other) = *ty {
                    unify.add_edge(node, other, *rule);
                }
            }
        }

        Session {
            nodes: Vec::from_iter(nodes),
            constraints,
            unify,
        }
    }
}

pub struct TyGroups(pub RefCell<BTreeMap<usize, RefCell<BTreeMap<NodeId, Option<AnyRule>>>>>);

impl Session {
    pub fn groups(&self) -> TyGroups {
        // Give every node a unique key to start
        let keys = self
            .unify
            .nodes()
            .enumerate()
            .map(|(index, node)| (node, index))
            .collect::<BTreeMap<_, _>>();

        // Merge keys that unify
        let mut union_find = UnionFind::new(keys.len());
        let mut rules = BTreeMap::new();
        for (left, right, rule) in self.unify.all_edges() {
            union_find.union(*keys.get(&left).unwrap(), *keys.get(&right).unwrap());
            let representative = union_find.find(*keys.get(&left).unwrap());
            rules.insert(representative, *rule);
        }

        // Create groups from the clustered keys
        let mut groups = vec![BTreeMap::<_, _>::new(); union_find.len()];
        for node in self.unify.nodes() {
            let index = *keys.get(&node).unwrap();
            let representative = union_find.find(index);
            let rule = *rules.get(&representative).unwrap();
            groups[representative].insert(node, Some(rule));
        }

        TyGroups(RefCell::new(
            groups
                .into_iter()
                .filter(|group| !group.is_empty())
                .enumerate()
                .map(|(index, group)| (index, RefCell::new(group)))
                .collect(),
        ))
    }

    pub fn iterate(&mut self, groups: &TyGroups) -> BTreeMap<NodeId, Vec<(Ty, Option<usize>)>> {
        let mut keys = groups
            .0
            .borrow()
            .iter()
            .flat_map(|(&key, group)| {
                group
                    .borrow()
                    .keys()
                    .map(move |&node| (node, key))
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeMap<_, _>>();

        let mut vars = vec![None; keys.len()];

        let key = |node: NodeId, vars: &mut Vec<_>, keys: &mut BTreeMap<_, _>| match keys.get(&node)
        {
            Some(key) => *key,
            None => {
                let key = keys.len();
                keys.insert(node, key);
                vars.push(None);
                key
            }
        };

        let narrow_key = |key: &mut usize, vars: &mut Vec<_>| {
            // Try to get a more specific key
            loop {
                let mut found = false;
                for (other, ty) in vars.iter().enumerate() {
                    if *ty == Some(Ty::Var(*key)) {
                        *key = other;
                        found = true;
                        break;
                    }
                }

                if !found {
                    break;
                }
            }
        };

        let instantiate = |ty: &mut Ty, vars: &mut Vec<_>, keys: &mut BTreeMap<_, _>| {
            ty.traverse_mut(&mut |ty| {
                if let Ty::Of(node) = *ty {
                    let mut key = key(node, vars, keys);
                    narrow_key(&mut key, vars);

                    *ty = Ty::Var(key);
                    ty.apply(vars);

                    // Add the node to its group with no specific rule
                    groups
                        .0
                        .borrow_mut()
                        .entry(key)
                        .or_default()
                        .borrow_mut()
                        .insert(node, None);
                }
            });
        };

        let group_ids = groups.0.borrow().keys().copied().collect::<Vec<_>>();

        let mut tys = self.constraints.tys.clone();
        let mut results = BTreeMap::<_, Vec<_>>::new();
        for group_id in group_ids {
            let nodes = groups
                .0
                .borrow()
                .get(&group_id)
                .unwrap()
                .borrow()
                .keys()
                .copied()
                .collect::<Vec<_>>();

            // Fold each type in the group into a single type
            let mut result_ty = Ty::Var(group_id);
            let mut others = Vec::new();
            for &node in &nodes {
                if let Some(tys) = tys.remove(&node) {
                    // Apply each constraint
                    for (mut ty, _) in tys {
                        instantiate(&mut ty, &mut vars, &mut keys);

                        let mut success = true;
                        let mut snapshot = result_ty.clone();
                        let mut vars_snapshot = vars.clone();

                        snapshot.unify_in_group(&ty, &mut vars_snapshot, &mut success);

                        if success {
                            result_ty = snapshot;
                            vars = vars_snapshot;
                        } else {
                            // No need to generate a diagnostic here; feedback
                            // is generated whenever an expression has multiple
                            // types
                            others.push(ty);
                        }
                    }
                }
            }

            // Share the result with every node in the group

            let result_tys = [result_ty]
                .into_iter()
                .chain(others.clone())
                .map(|ty| (ty, Some(group_id)))
                .collect::<Vec<_>>();

            for &node in &nodes {
                results.entry(node).or_default().extend(result_tys.clone());
            }
        }

        // Any remaining constraints weren't part of groups
        for (node, constraints) in mem::take(&mut tys) {
            for (mut ty, _) in constraints {
                instantiate(&mut ty, &mut vars, &mut keys);
                results.entry(node).or_default().push((ty, None));
            }
        }

        // Include all nodes, including those that had no constraints at all,
        // and patch up groups
        for &node in &self.nodes {
            let Some(mut key) = keys.get(&node).copied() else {
                continue;
            };

            narrow_key(&mut key, &mut vars);

            // Add the node to its group with no specific rule
            groups
                .0
                .borrow_mut()
                .entry(key)
                .or_default()
                .borrow_mut()
                .insert(node, None);

            // Add at least the node's own type
            results.entry(node).or_insert_with(|| {
                // The type is essentially a placeholder; no rules here
                vec![(Ty::Var(key), Some(key))]
            });
        }

        // Apply all types
        for group in results.values_mut() {
            for (ty, _) in group {
                ty.apply(&vars);
            }
        }

        results
    }
}

impl Ty {
    fn unify_in_group(&mut self, other: &Ty, vars: &mut Vec<Option<Ty>>, success: &mut bool) {
        self.apply(vars);

        match (self, other) {
            (ty @ &mut Ty::Var(key), other) => {
                let mut other = other.clone();
                other.apply(vars);

                match vars[key].clone() {
                    Some(mut ty) => {
                        ty.unify_in_group(&other, vars, success);
                    }
                    None => {
                        // TODO: Check recursively here if necessary
                        if other != Ty::Var(key) {
                            vars[key] = Some(other.clone());
                            *ty = other.clone();
                            ty.apply(vars);
                        }
                    }
                }
            }
            (ty, &Ty::Var(key)) => {
                match vars[key].clone() {
                    Some(other) => {
                        ty.unify_in_group(&other, vars, success);
                    }
                    None => {
                        // TODO: Check recursively here if necessary
                        if *ty != Ty::Var(key) {
                            vars[key] = Some(ty.clone());
                            ty.apply(vars);
                        }
                    }
                }
            }
            (_, Ty::Of(..)) | (Ty::Of(..), _) => {
                unreachable!("`Ty::Of` should be replaced with `Ty::Var`")
            }
            (
                Ty::Named { name, parameters },
                Ty::Named {
                    name: other_name,
                    parameters: other_parameters,
                },
            ) if name == other_name => {
                for (parameter, other) in parameters.iter_mut().zip(other_parameters) {
                    parameter.unify_in_group(other, vars, success);
                }
            }
            (
                Ty::Function { inputs, output },
                Ty::Function {
                    inputs: other_inputs,
                    output: other_output,
                },
            ) if inputs.len() == other_inputs.len() => {
                for (input, other) in inputs.iter_mut().zip(other_inputs) {
                    input.unify_in_group(other, vars, success);
                }

                output.unify_in_group(other_output, vars, success);
            }
            (
                Ty::Tuple { elements },
                Ty::Tuple {
                    elements: other_elements,
                },
            ) if elements.len() == other_elements.len() => {
                for (element, other) in elements.iter_mut().zip(other_elements) {
                    element.unify_in_group(other, vars, success);
                }
            }
            _ => *success = false,
        }
    }

    fn apply(&mut self, vars: &[Option<Ty>]) {
        self.traverse_mut(&mut |ty| {
            if let Ty::Var(key) = *ty {
                if let Some(other) = vars[key].clone() {
                    *ty = other;
                    ty.apply(vars);
                }
            }
        });
    }
}

impl Session {
    pub fn to_debug_graph(
        &self,
        groups: &TyGroups,
        tys: &BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
        relations: &DiGraphMap<NodeId, AnyRule>,
        provider: &FeedbackProvider<'_>,
    ) -> String {
        let font = "Fira Code";

        let success_color = |base: u8| tabbycat::attributes::Color::Rgb(base, base, 255);
        let error_color = |base: u8| tabbycat::attributes::Color::Rgb(255, base, base);

        let node_id = |node: NodeId| tabbycat::Identity::raw(format!("node{}", node.0));

        let mut stmts = tabbycat::StmtList::new();

        for (node, rule) in groups
            .0
            .borrow()
            .values()
            .flat_map(|group| {
                group
                    .borrow()
                    .iter()
                    .map(|(&node, &rule)| (node, rule))
                    .collect::<Vec<_>>()
            })
            .chain(self.nodes.iter().map(|&node| (node, None)))
            .unique_by(|&(node, _)| node)
        {
            let (node_span, node_source) = provider.node_span_source(node);

            let label = match rule {
                Some(rule) => {
                    format!("{node_span:?}\n{node_source}\n----------------\n{rule:?}")
                }
                None => format!("{node_span:?}\n{node_source}"),
            };

            stmts = mem::take(&mut stmts).add_node(
                node_id(node),
                None,
                Some(
                    tabbycat::AttrList::new()
                        .add_pair(tabbycat::attributes::label(label))
                        .add_pair(tabbycat::attributes::shape(
                            tabbycat::attributes::Shape::Box,
                        ))
                        .add_pair(tabbycat::attributes::width(3.))
                        .add_pair(tabbycat::attributes::fontname(font)),
                ),
            );

            // Also link related nodes
            for parent in relations.neighbors_directed(node, Direction::Incoming) {
                let &rule = relations.edge_weight(parent, node).unwrap();

                if rule.kind().is_hidden() {
                    continue;
                }

                stmts = stmts.add_edge(
                    tabbycat::Edge::head_node(node_id(node), None)
                        .arrow_to_node(node_id(parent), None)
                        .add_attrpair(tabbycat::attributes::label(format!("{rule:?}")))
                        .add_attrpair(tabbycat::attributes::fontname(font))
                        .add_attrpair(tabbycat::attributes::style(
                            tabbycat::attributes::Style::Solid,
                        )),
                );
            }
        }

        for (&id, group) in groups.0.borrow().iter() {
            let group_tys = group
                .borrow()
                .keys()
                .flat_map(|node| tys.get(node).unwrap())
                .map(|(ty, _)| ty)
                .unique()
                .collect::<Vec<_>>();

            let error = group_tys.len() > 1; // mutiple possible types

            let color = if error { error_color } else { success_color };

            let group_tys = group_tys
                .iter()
                .map(|ty| ty.to_debug_string(provider))
                .collect::<Vec<_>>()
                .join("\n");

            let mut attrs = tabbycat::AttrList::new()
                .add_pair(tabbycat::attributes::style(
                    tabbycat::attributes::Style::Dashed,
                ))
                .add_pair(tabbycat::attributes::label(group_tys))
                .add_pair(tabbycat::attributes::fontname(format!("{font} Semibold")))
                .add_pair(tabbycat::attributes::fontcolor(color(0)));

            attrs = attrs
                .add_pair(tabbycat::attributes::bgcolor(color(240)))
                .add_pair(tabbycat::attributes::color(color(0)));

            stmts = stmts.add_subgraph(tabbycat::SubGraph::subgraph(
                Some(tabbycat::Identity::raw(format!("cluster{id}"))),
                group.borrow().iter().fold(
                    tabbycat::StmtList::new().add_attr(tabbycat::AttrType::Graph, attrs),
                    |stmts, (&node, _)| stmts.add_node(node_id(node), None, None),
                ),
            ));
        }

        // Deduplicate statements
        let stmts = tabbycat::StmtList::new()
            .add_equation(
                tabbycat::Identity::raw("rankdir"),
                tabbycat::Identity::quoted("LR"),
            )
            .extend(stmts.into_iter().unique_by(|stmt| stmt.to_string()));

        tabbycat::GraphBuilder::default()
            .id(tabbycat::Identity::quoted("output"))
            .strict(false)
            .graph_type(tabbycat::GraphType::DiGraph)
            .stmts(stmts)
            .build()
            .unwrap()
            .to_string()
    }
}
