use crate::constraints::{Constraints, Ty};
use petgraph::prelude::UnGraphMap;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    mem,
};
use wipple_compiler_trace::NodeId;

pub struct Typechecker<'a> {
    pub constraints: Constraints,
    groups: Groups,
    instantiated: BTreeMap<(NodeId, NodeId), usize>,
    _temp: std::marker::PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
struct Groups(Vec<Group>);

impl std::ops::Deref for Groups {
    type Target = Vec<Group>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Groups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Group {
    ty: Option<Ty>,
    nodes: BTreeSet<NodeId>,
}

impl Group {
    fn new(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Group {
            ty: None,
            nodes: BTreeSet::from_iter(nodes),
        }
    }
}

impl<'a> Typechecker<'a> {
    pub fn from_constraints(
        nodes: impl IntoIterator<Item = NodeId>,
        constraints: Constraints,
    ) -> Self {
        // Form initial groups based on nodes that directly relate to each other
        let mut directly_related = UnGraphMap::<NodeId, ()>::new();
        for (&node, tys) in &constraints.tys {
            for (ty, _) in tys {
                // We don't need to deeply traverse the type because all types
                // here represent the top-level type of a node. For example, if
                // we have `x :: (1) -> _` and `y :: (1) -> _`, then at this
                // point the (1) must be attached to another specific node in
                // the AST (which will be handled in another iteration), or it
                // won't influence other nodes anyway.
                if let Ty::Of(other) = *ty {
                    directly_related.add_edge(node, other, ());
                }
            }
        }

        // "Groups" are the connected components of the graph
        let mut groups = petgraph::algo::tarjan_scc(&directly_related)
            .into_iter()
            .map(Group::new)
            .collect::<Vec<_>>();

        // Put nodes with no relations into their own groups to start with
        for node in nodes {
            if !directly_related.contains_node(node) {
                groups.push(Group::new([node]));
            }
        }

        Typechecker {
            constraints,
            groups: Groups(groups),
            instantiated: Default::default(),
            _temp: std::marker::PhantomData,
        }
    }

    pub fn run(self) -> Constraints {
        self.run_until(None).unwrap_or_else(|_| unreachable!())
    }

    pub fn run_until(mut self, limit: impl Into<Option<usize>>) -> Result<Constraints, Self> {
        let limit = limit.into();

        let mut counter = 0;
        while limit.is_none_or(|limit| counter < limit) {
            counter += 1;

            if self.iterate_tys() {
                continue;
            }

            // TODO: If no progress, apply bounds; still no progress, apply
            // defaults; etc.

            return Ok(self.constraints);
        }

        // Reached limit
        Err(self)
    }
}

impl Typechecker<'_> {
    fn iterate_tys(&mut self) -> bool {
        let mut tys = self.constraints.tys.clone();

        let mut queue = VecDeque::from_iter(0..self.groups.len());

        // Unify types within groups
        let mut results = BTreeMap::<_, Vec<_>>::new();
        while let Some(mut index) = queue.pop_front() {
            let nodes_snapshot = self.groups[index].nodes.clone();

            let mut others = BTreeMap::new();
            for node in nodes_snapshot {
                for (mut ty, _) in tys.remove(&node).unwrap_or_default() {
                    self.groups.replace_of_with_group(&mut ty);

                    let mut success = true;
                    let mut groups_snapshot = self.groups.clone();

                    let mut merge_group = |_from_index, to_index| {
                        queue.push_back(to_index);
                    };

                    Ty::Group(index).unify_with(
                        &ty,
                        &mut groups_snapshot,
                        &mut success,
                        &mut merge_group,
                    );

                    if success {
                        self.groups = groups_snapshot;
                    } else {
                        // No need to generate a diagnostic here; feedback
                        // is generated whenever an expression has multiple
                        // types
                        others.insert(node, ty);
                    }
                }
            }

            self.groups.apply_index(&mut index);

            // Share the group's final type with every node in the group

            let group_ty = self.groups[index].ty.clone().unwrap_or(Ty::Group(index));

            for &node in &self.groups[index].nodes {
                results
                    .entry(node)
                    .or_default()
                    .push((group_ty.clone(), Some(index)));

                if let Some(other) = others.remove(&node) {
                    results.entry(node).or_default().push((other, Some(index)));
                }
            }
        }

        // Any remaining constraints weren't part of groups
        for (&node, constraints) in &mut tys {
            for (ty, _) in constraints {
                self.groups.replace_of_with_group(ty);
                results.entry(node).or_default().push((ty.clone(), None));
            }
        }

        // Apply all types
        for tys in results.values_mut() {
            for (ty, index) in tys {
                ty.apply(&mut self.groups);

                if let Some(index) = index {
                    self.groups.apply_index(index);
                } else {
                    // Will be added to a group next time
                }
            }
        }

        // TODO: Set to `true` when a specific change is made rather than
        // checking everything
        let progress = self.constraints.tys != results;

        self.constraints.tys = results;

        progress
    }
}

impl Groups {
    fn apply_index(&mut self, index: &mut usize) {
        let mut seen = vec![*index];
        loop {
            let mut found = false;
            for (other_index, group) in self.iter().enumerate() {
                if seen.contains(&other_index) {
                    continue;
                }

                if group.ty == Some(Ty::Group(*index)) {
                    *index = other_index;
                    seen.push(other_index);
                    found = true;
                    break;
                }
            }

            if !found {
                break;
            }
        }
    }

    fn new_index(&mut self, node: Option<NodeId>) -> usize {
        let index = self.len();
        self.push(Group::new(node));
        index
    }

    fn try_index_for(&self, node: NodeId) -> Option<usize> {
        let candidates = self
            .iter()
            .enumerate()
            .filter(|(_, group)| group.nodes.contains(&node))
            .collect::<Vec<_>>();

        match candidates.as_slice() {
            [] => None,
            [(index, _)] => Some(*index),
            _ => panic!("node {node:?}, belongs to multiple groups: {candidates:?}"),
        }
    }

    fn index_for(&mut self, node: Option<NodeId>) -> usize {
        node.and_then(|node| {
            let mut index = self.try_index_for(node)?;
            self.apply_index(&mut index);
            Some(index)
        })
        .unwrap_or_else(|| self.new_index(node))
    }

    fn replace_of_with_group(&mut self, ty: &mut Ty) {
        ty.apply(self);
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = *ty {
                *ty = Ty::Group(self.index_for(Some(node)));
                ty.apply(self);
            }
        });
    }
}

impl Ty {
    fn unify_with(
        &mut self,
        other: &Ty,
        groups: &mut Groups,
        success: &mut bool,
        merge_group: &mut impl FnMut(usize, usize),
    ) {
        self.apply(groups);

        match (self, other) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => {}
            (_, Ty::Of(..)) | (Ty::Of(..), _) => {
                unreachable!("`Ty::Of` should be replaced with `Ty::Group`")
            }
            (Ty::Group(index), Ty::Group(other_index)) => {
                let Ok([group, other_group]) = groups.get_disjoint_mut([*index, *other_index])
                else {
                    // Indices are the same; do nothing
                    return;
                };

                // We don't need to unify the other group's type with the
                // current group's, because if this arm is reached, neither
                // group actually has a type yet (since types are applied
                // beforehand).
                assert!(group.ty.is_none());
                assert!(other_group.ty.is_none());

                group.nodes.extend(mem::take(&mut other_group.nodes));
                merge_group(*index, *other_index);
            }
            (ty @ &mut Ty::Group(index), other) => {
                let mut other = other.clone();
                other.apply(groups);

                assert!(groups[index].ty.is_none());
                groups[index].ty = Some(other.clone());

                ty.apply(groups);
            }
            (other, &Ty::Group(index)) => {
                other.apply(groups);

                assert!(groups[index].ty.is_none());
                groups[index].ty = Some(other.clone());
            }
            (
                Ty::Named { name, parameters },
                Ty::Named {
                    name: other_name,
                    parameters: other_parameters,
                },
            ) if name == other_name => {
                for (parameter, other) in parameters.iter_mut().zip(other_parameters) {
                    parameter.unify_with(other, groups, success, merge_group);
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
                    input.unify_with(other, groups, success, merge_group);
                }

                output.unify_with(other_output, groups, success, merge_group);
            }
            (
                Ty::Tuple { elements },
                Ty::Tuple {
                    elements: other_elements,
                },
            ) if elements.len() == other_elements.len() => {
                for (element, other) in elements.iter_mut().zip(other_elements) {
                    element.unify_with(other, groups, success, merge_group);
                }
            }
            _ => *success = false,
        }
    }

    fn apply(&mut self, groups: &mut Groups) {
        self.apply_inner(groups, &mut Vec::new());
    }

    fn apply_inner(&mut self, groups: &mut Groups, stack: &mut Vec<usize>) {
        self.traverse_mut(&mut |ty| {
            if let Ty::Group(mut index) = *ty {
                groups.apply_index(&mut index);

                if stack.contains(&index) {
                    panic!(
                        "cycle detected: {}{}",
                        stack
                            .iter()
                            .map(|index| format!("{index} -> "))
                            .collect::<String>(),
                        index
                    );
                }

                if let Some(group_ty) = &groups[index].ty {
                    stack.push(index);

                    *ty = group_ty.clone();
                    ty.apply_inner(groups, stack);

                    stack.pop();
                }
            }
        });
    }

    // fn instantiate(&mut self, node: NodeId, definition: NodeId, typechecker: &mut Typechecker<'_>) {
    //     *self = Ty::Of(definition);
    // }
}
