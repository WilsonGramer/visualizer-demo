use crate::constraints::{Constraints, Ty};
use petgraph::{prelude::UnGraphMap, unionfind::UnionFind};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    mem,
};
use wipple_compiler_trace::NodeId;

#[derive(Clone)]
pub struct Session {
    pub(crate) nodes: Vec<NodeId>,
    pub(crate) constraints: Constraints,
    pub(crate) unify: UnGraphMap<NodeId, ()>,
}

impl Session {
    pub fn from_constraints(
        nodes: impl IntoIterator<Item = NodeId>,
        constraints: Constraints,
    ) -> Self {
        let mut unify = UnGraphMap::<NodeId, ()>::new();
        for (&node, tys) in &constraints.tys {
            for ty in tys {
                // We don't need to deeply traverse the type because all types
                // here represent the top-level type of a node. For example, if
                // we have `x :: (1) -> _` and `y :: (1) -> _`, then at this
                // point the (1) must be attached to another specific node in
                // the AST (which will be handled in another iteration), or it
                // can't be inferred anyway.
                if let Ty::Of(other) = *ty {
                    unify.add_edge(node, other, ());
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

pub struct TyGroups(pub RefCell<BTreeMap<usize, RefCell<BTreeSet<NodeId>>>>);

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
        for (left, right, _) in self.unify.all_edges() {
            union_find.union(*keys.get(&left).unwrap(), *keys.get(&right).unwrap());
        }

        // Create groups from the clustered keys
        let mut groups = vec![BTreeSet::<_>::new(); union_find.len()];
        for node in self.unify.nodes() {
            let index = *keys.get(&node).unwrap();
            let representative = union_find.find(index);
            groups[representative].insert(node);
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
                    .iter()
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
                        .insert(node);
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
                .iter()
                .copied()
                .collect::<Vec<_>>();

            // Fold each type in the group into a single type
            let mut result_ty = Ty::Var(group_id);
            let mut others = Vec::new();
            for &node in &nodes {
                if let Some(tys) = tys.remove(&node) {
                    // Apply each constraint
                    for mut ty in tys {
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
            for mut ty in constraints {
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
                .insert(node);

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
