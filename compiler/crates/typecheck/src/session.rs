use crate::{
    constraints::{Constraint, Constraints, Ty},
    context::{DebugOptions, DebugProvider},
};
use itertools::Itertools;
use petgraph::{
    Direction,
    prelude::{DiGraphMap, UnGraphMap},
    unionfind::UnionFind,
    visit::{DfsEvent, depth_first_search},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    mem,
};
use wipple_compiler_trace::{AnyRule, NodeId, Span, rule};

rule! {
    /// The type was unified with another type.
    unified;
}

#[derive(Clone)]
pub struct Session {
    node_constraints: BTreeMap<NodeId, Vec<Constraint>>,

    /// The type of node _a_ is influenced by the type of node _b_, but these
    /// types do NOT necessarily unify. Used for tracing.
    influencing_nodes: DiGraphMap<NodeId, AnyRule>,

    /// The type of node _a_ unifies with the type of node _b_.
    // TODO: Factor out into `Iteration`
    unify_nodes: UnGraphMap<NodeId, ()>,
}

impl Session {
    pub fn new(constraints: &Constraints) -> Self {
        let mut node_constraints = BTreeMap::<NodeId, Vec<_>>::new();

        let mut influencing_nodes = DiGraphMap::new();
        for (&node, node_tys) in &constraints.tys {
            for (ty, rule) in node_tys {
                // Add the overall type...
                node_constraints
                    .entry(node)
                    .or_default()
                    .push(Constraint::Ty(ty.clone()));

                // ...and record parts that are influenced by other nodes
                ty.traverse(&mut |ty| {
                    if let Ty::Of(other, direction) = *ty {
                        // The direction is inverted so we can search for all
                        // influencing nodes using DFS
                        match direction {
                            Direction::Incoming => {
                                influencing_nodes.add_edge(node, other, *rule);
                            }
                            Direction::Outgoing => {
                                influencing_nodes.add_edge(other, node, *rule);
                            }
                        }
                    }
                });
            }
        }

        for (&node, bounds) in &constraints.bounds {
            for (bound, rule) in bounds {
                node_constraints
                    .entry(node)
                    .or_default()
                    .push(Constraint::Bound(bound.clone()));

                influencing_nodes.add_edge(node, bound.source, *rule);
            }
        }

        let mut unify_nodes = UnGraphMap::<NodeId, ()>::new();

        for (&node, node_tys) in &constraints.tys {
            // Ensure all nodes are at least added to the graph so they can be
            // placed in a group
            unify_nodes.add_node(node);

            // Try every combination of types, rather than folding from left to
            // right, to create the largest potential number of groups. Include
            // the node type itself in the list of combinations in case there's
            // only one additional type
            for (left, right) in [&Ty::Of(node, Direction::Incoming)]
                .into_iter()
                .chain(node_tys.iter().map(|(ty, _)| ty))
                .tuple_combinations()
            {
                let mut unify_nodes_snapshot = unify_nodes.clone();
                let mut success = true;
                left.unify_to_keys(right, &mut unify_nodes_snapshot, &mut success);

                if success {
                    unify_nodes = unify_nodes_snapshot;
                }
            }
        }

        Session {
            node_constraints,
            influencing_nodes,
            unify_nodes,
        }
    }

    pub fn groups(&self, mask: Option<&BTreeSet<NodeId>>) -> Vec<Vec<NodeId>> {
        // Give every node a unique key to start
        let keys = self
            .unify_nodes
            .nodes()
            .filter(|node| mask.is_none_or(|mask| mask.contains(node)))
            .enumerate()
            .map(|(index, node)| (node, index))
            .collect::<BTreeMap<_, _>>();

        // Merge keys that unify
        let mut union_find = UnionFind::new(keys.len());
        for (left, right, _) in self.unify_nodes.all_edges() {
            if mask.is_none_or(|mask| mask.contains(&left) && mask.contains(&right)) {
                union_find.union(*keys.get(&left).unwrap(), *keys.get(&right).unwrap());
            }
        }

        // Create groups from the clustered keys
        let mut groups = vec![Vec::new(); union_find.len()];
        for node in self.unify_nodes.nodes() {
            if mask.is_none_or(|mask| mask.contains(&node)) {
                let index = *keys.get(&node).unwrap();
                let group = union_find.find(index);
                groups[group].push(node);
            }
        }

        groups
            .into_iter()
            .filter(|group| !group.is_empty())
            .collect()
    }

    pub fn iterate(&mut self, debug: &DebugProvider<'_>) -> BTreeMap<NodeId, Vec<Ty>> {
        let groups = self.groups(None);

        let mut keys = groups
            .iter()
            .enumerate()
            .flat_map(|(index, group)| group.iter().map(move |&node| (node, index)))
            .collect::<BTreeMap<_, _>>();

        let mut vars = vec![None; keys.len()];

        let mut key = |node: NodeId, vars: &mut Vec<_>| match keys.get(&node) {
            Some(key) => *key,
            None => {
                let key = keys.len();
                keys.insert(node, key);
                vars.push(None);
                key
            }
        };

        let mut instantiate = |ty: &mut Ty, vars: &mut Vec<_>| {
            ty.traverse_mut(&mut |ty| {
                if let Ty::Of(node, _) = ty {
                    *ty = Ty::Var(key(*node, vars));
                }
            });
        };

        let mut tys = BTreeMap::<_, Vec<_>>::new();
        let mut other_constraints = BTreeMap::<_, Vec<_>>::new();
        for (index, group) in groups.into_iter().enumerate() {
            // Fold each type in the group into a single type
            let mut result = Ty::Var(index);
            let mut others = Vec::new();
            for &node in &group {
                if let Some(constraints) = self.node_constraints.remove(&node) {
                    for constraint in constraints {
                        match constraint {
                            Constraint::Ty(mut ty) => {
                                instantiate(&mut ty, &mut vars);

                                let mut success = true;
                                let mut snapshot = result.clone();
                                let mut vars_snapshot = vars.clone();
                                snapshot.unify_in_group(&ty, &mut vars_snapshot, &mut success);

                                if success {
                                    result = snapshot;
                                    vars = vars_snapshot;
                                } else {
                                    // TODO: Diagnostic here?
                                    ty.apply(&vars_snapshot);
                                    eprintln!(
                                        "failed to unify {} with {}",
                                        ty.to_debug_string(debug),
                                        snapshot.to_debug_string(debug),
                                    );

                                    others.push(ty);
                                }
                            }
                            constraint @ Constraint::Bound(..) => {
                                other_constraints
                                    .entry(node)
                                    .or_default()
                                    .push(constraint.clone());
                            }
                        }
                    }
                }
            }

            // Share the result with every node in the group

            let result = [result.clone()]
                .into_iter()
                .chain(others.clone())
                .collect::<Vec<_>>();

            for &node in &group {
                tys.entry(node).or_default().extend(result.clone());
            }
        }

        // Any remaining constraints weren't part of groups
        for (node, constraints) in mem::take(&mut self.node_constraints) {
            for constraint in constraints {
                match constraint {
                    Constraint::Ty(mut ty) => {
                        instantiate(&mut ty, &mut vars);
                        tys.entry(node).or_default().push(ty);
                    }
                    Constraint::Bound(_) => {
                        other_constraints
                            .entry(node)
                            .or_default()
                            .push(constraint.clone());
                    }
                }
            }
        }

        // Add at least the node's own type
        for node in self.influencing_nodes.nodes() {
            tys.entry(node).or_insert_with(|| {
                let mut ty = Ty::Var(key(node, &mut vars));
                ty.apply(&vars);

                // The type is essentially a placeholder; no rules here
                vec![ty]
            });
        }

        // Apply all types
        for tys in tys.values_mut() {
            for ty in tys {
                ty.apply(&vars);
            }
        }

        // Merge the new types back into the constraints

        self.node_constraints.extend(tys.iter().map(|(&node, tys)| {
            // Keep only the primary `result` in the constraints...
            let ty = tys.iter().next().unwrap().clone();

            (node, vec![Constraint::Ty(ty)])
        }));

        self.node_constraints.extend(other_constraints);

        // ...but return all possible types for diagnostics
        tys
    }
}

impl Ty {
    /// Before actually unifying, connect all related nodes to a single type
    /// variable.
    fn unify_to_keys(
        &self,
        other: &Ty,
        unify_nodes: &mut UnGraphMap<NodeId, ()>,
        success: &mut bool,
    ) {
        match (self, other) {
            (&Ty::Of(node, _), &Ty::Of(other, _)) => {
                unify_nodes.add_edge(node, other, ());
            }
            (Ty::Var(_), _) | (_, Ty::Var(_)) => {
                unreachable!("this runs before creating type variables")
            }
            (
                Ty::Named { name, parameters },
                Ty::Named {
                    name: other_name,
                    parameters: other_parameters,
                },
            ) if name == other_name => {
                for (parameter, other) in parameters.iter().zip(other_parameters) {
                    parameter.unify_to_keys(other, unify_nodes, success);
                }
            }
            (
                Ty::Function { inputs, output },
                Ty::Function {
                    inputs: other_inputs,
                    output: other_output,
                },
            ) if inputs.len() == other_inputs.len() => {
                for (input, other) in inputs.iter().zip(other_inputs) {
                    input.unify_to_keys(other, unify_nodes, success);
                }

                output.unify_to_keys(other_output, unify_nodes, success);
            }
            (
                Ty::Tuple { elements },
                Ty::Tuple {
                    elements: other_elements,
                },
            ) if elements.len() == other_elements.len() => {
                for (element, other) in elements.iter().zip(other_elements) {
                    element.unify_to_keys(other, unify_nodes, success);
                }
            }
            _ => *success = false,
        }
    }

    fn unify_in_group(&mut self, other: &Ty, vars: &mut Vec<Option<Ty>>, success: &mut bool) {
        self.apply(vars);

        match (self, other) {
            (ty @ &mut Ty::Var(key), other) => {
                let mut other = other.clone();
                other.apply(vars);

                if other != Ty::Var(key) {
                    match vars[key].clone() {
                        Some(other) => {
                            ty.unify_in_group(&other, vars, success);
                        }
                        None => {
                            vars[key] = Some(other.clone());
                            *ty = other.clone();
                        }
                    }
                }

                ty.apply(vars);
            }
            (ty, &Ty::Var(key)) => {
                if *ty != Ty::Var(key) {
                    match vars[key].clone() {
                        Some(other) => {
                            ty.unify_in_group(&other, vars, success);
                        }
                        None => {
                            vars[key] = Some(ty.clone());
                            ty.apply(vars);
                        }
                    }
                }

                ty.apply(vars);
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
                }
            }
        });
    }
}

impl Session {
    pub fn to_debug_graph(
        &self,
        start: Option<NodeId>,
        tys: &BTreeMap<NodeId, Vec<Ty>>,
        relations: &BTreeMap<NodeId, (NodeId, AnyRule)>,
        debug: &DebugProvider<'_>,
    ) -> String {
        // Group types that should unify with each other

        let edges = match start {
            Some(start) => {
                let mut edges = BTreeMap::new();
                depth_first_search(&self.influencing_nodes, [start], |event| {
                    if let DfsEvent::TreeEdge(to, from) = event {
                        edges.insert(
                            (to, from),
                            Some(*self.influencing_nodes.edge_weight(to, from).unwrap()),
                        );

                        depth_first_search(&self.unify_nodes, [to, from], |event| {
                            if let DfsEvent::TreeEdge(to, from) = event {
                                edges.insert((to, from), None);
                            }
                        });
                    }
                });

                edges
            }
            None => self
                .influencing_nodes
                .all_edges()
                .map(|(to, from, rule)| ((to, from), Some(*rule)))
                .collect(),
        };

        let mask = start.map(|_| {
            edges
                .keys()
                .flat_map(|&(to, from)| [to, from])
                .collect::<BTreeSet<_>>()
        });

        let groups = self.groups(mask.as_ref());

        // Output Graphviz

        let font = "Fira Code";
        let error_color =
            |opacity: f32| tabbycat::attributes::Color::Rgba(255, 0, 0, (opacity * 255.) as u8);

        let node_id = |node: NodeId| tabbycat::Identity::raw(format!("node{}", node.0));

        let mut stmts = tabbycat::StmtList::new();

        for (index, group) in groups.iter().enumerate() {
            let tys = tys.get(&group[0]).unwrap();
            let error = tys.len() > 1;

            let mut attrs = tabbycat::AttrList::new().add_pair(tabbycat::attributes::style(
                tabbycat::attributes::Style::Dashed,
            ));

            if error {
                attrs = attrs
                    .add_pair(tabbycat::attributes::bgcolor(error_color(0.1)))
                    .add_pair(tabbycat::attributes::color(error_color(1.)));
            }

            stmts = stmts.add_subgraph(tabbycat::SubGraph::subgraph(
                Some(tabbycat::Identity::raw(format!("cluster{index}"))),
                group
                    .iter()
                    .fold(tabbycat::StmtList::new(), |stmts, node| {
                        stmts.add_edge(tabbycat::Edge::head_node(node_id(*node), None))
                    })
                    .add_attr(tabbycat::AttrType::Graph, attrs),
            ))
        }

        for ((to, from), rule) in edges {
            let Some(rule) = rule else {
                // `None` is a marker for unified types; include the node in the
                // graph, but don't add an edge
                continue;
            };

            let display_tys = |node: NodeId| {
                tys.get(&node)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .map(|ty| format!("\n{}", ty.to_debug_string(debug)))
                    .unique()
                    .collect::<String>()
            };

            let from_tys = display_tys(from);
            let to_tys = display_tys(to);

            let (from_span, from_debug) = debug.node(
                from,
                DebugOptions {
                    rule: true,
                    ..Default::default()
                },
            );

            let (to_span, to_debug) = debug.node(
                to,
                DebugOptions {
                    rule: true,
                    ..Default::default()
                },
            );

            stmts = mem::take(&mut stmts)
                .add_node(
                    node_id(from),
                    None,
                    Some(
                        tabbycat::AttrList::new()
                            .add_pair(tabbycat::attributes::label(format!(
                                "{from_span:?}\n{from_debug}\n----------------{from_tys}"
                            )))
                            .add_pair(tabbycat::attributes::shape(
                                tabbycat::attributes::Shape::Box,
                            ))
                            .add_pair(tabbycat::attributes::width(3.))
                            .add_pair(tabbycat::attributes::fontname(font)),
                    ),
                )
                .add_node(
                    node_id(to),
                    None,
                    Some(
                        tabbycat::AttrList::new()
                            .add_pair(tabbycat::attributes::label(format!(
                                "{to_span:?}\n{to_debug}\n----------------{to_tys}"
                            )))
                            .add_pair(tabbycat::attributes::shape(
                                tabbycat::attributes::Shape::Box,
                            ))
                            .add_pair(tabbycat::attributes::width(3.))
                            .add_pair(tabbycat::attributes::fontname(font)),
                    ),
                )
                .add_edge(
                    tabbycat::Edge::head_node(node_id(from), None)
                        .arrow_to_node(node_id(to), None)
                        .add_attrpair(tabbycat::attributes::label(format!("{rule:?}")))
                        .add_attrpair(tabbycat::attributes::fontname(font))
                        .add_attrpair(tabbycat::attributes::style(
                            tabbycat::attributes::Style::Dashed,
                        )),
                );

            for child in [from, to] {
                if let Some((parent, rule)) = relations.get(&child) {
                    stmts = stmts.add_edge(
                        tabbycat::Edge::head_node(node_id(child), None)
                            .arrow_to_node(node_id(*parent), None)
                            .add_attrpair(tabbycat::attributes::label(format!("{rule:?}")))
                            .add_attrpair(tabbycat::attributes::fontname(font))
                            .add_attrpair(tabbycat::attributes::style(
                                tabbycat::attributes::Style::Solid,
                            )),
                    );
                }
            }
        }

        // Deduplicate statements
        let stmts =
            tabbycat::StmtList::new().extend(stmts.into_iter().unique_by(|stmt| stmt.to_string()));

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
