use crate::constraints::{Constraints, Ty, TyConstraints};
use petgraph::prelude::UnGraphMap;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct Session {
    pub constraints: Constraints,
    groups: Vec<Group>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Session {
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

        Session {
            constraints,
            groups,
        }
    }

    pub fn run(&mut self) -> TyConstraints {
        // FIXME: Run multiple times
        self.next().unwrap_or_default()
    }
}

impl Session {
    fn refine_index(&mut self, index: &mut usize) {
        loop {
            let mut found = false;
            for (other_index, group) in self.groups.iter().enumerate() {
                if group.ty == Some(Ty::Group(*index)) && *index != other_index {
                    *index = other_index;
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
        let index = self.groups.len();
        self.groups.push(Group::new(node));
        index
    }

    fn index_of(&self, node: NodeId) -> Option<usize> {
        self.groups
            .iter()
            .position(|group| group.nodes.contains(&node))
    }

    fn index_for(&mut self, node: Option<NodeId>) -> usize {
        let mut index = node
            .and_then(|node| self.index_of(node))
            .unwrap_or_else(|| self.new_index(node));

        self.refine_index(&mut index);

        index
    }

    fn replace_of_with_group(&mut self, ty: &mut Ty) {
        ty.apply(self);
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = ty {
                let index = self.index_for(Some(*node));
                *ty = Ty::Group(index);
                ty.apply(self);
            }
        });
    }
}

impl Iterator for Session {
    type Item = TyConstraints;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tys = self.constraints.tys.clone();

        let mut groups_snapshot = self
            .groups
            .iter()
            .cloned()
            .enumerate() // must be done before sorting
            .collect::<Vec<_>>();

        // Form better groups by applying groups containing incomplete types first
        groups_snapshot.sort_by_key(|(_, group)| {
            group
                .nodes
                .iter()
                .flat_map(|node| {
                    tys.get(node)
                        .into_iter()
                        .flatten()
                        .map(|(ty, _)| {
                            let mut ty = ty.clone();
                            ty.apply(self);
                            ty.relative_ordering()
                        })
                        .min()
                })
                .min()
        });

        // Unify types within groups
        let mut results = BTreeMap::<_, Vec<_>>::new();
        for &(index, ref group) in &groups_snapshot {
            let tys = group
                .nodes
                .iter()
                .filter_map(|&node| Some(tys.get(&node)?.iter().cloned().map(move |ty| (node, ty))))
                .flatten()
                .collect::<Vec<_>>();

            // Fold each type in the group into a single type
            let mut others = Vec::new();
            for (_node, (mut ty, _)) in tys {
                if ty.needs_instantiation() {
                    others.push(ty);
                    continue;
                }

                self.replace_of_with_group(&mut ty);

                let mut success = true;
                Ty::Group(index).unify_with(&ty, self, &mut success);

                if !success {
                    // No need to generate a diagnostic here; feedback
                    // is generated whenever an expression has multiple
                    // types
                    others.push(ty);
                }
            }

            // Share the result with every node in the group

            let result_tys = [Ty::Group(index)]
                .into_iter()
                .chain(others.clone())
                .map(|ty| (ty, Some(index)))
                .collect::<Vec<_>>();

            for &node in &group.nodes {
                results.entry(node).or_default().extend(result_tys.clone());
            }
        }

        // Any remaining constraints weren't part of groups
        for (node, constraints) in &mut tys {
            for (ty, _) in constraints {
                if ty.needs_instantiation() {
                    continue;
                }

                self.replace_of_with_group(ty);
                results.entry(*node).or_default().push((ty.clone(), None));
            }
        }

        // Apply all types
        for tys in results.values_mut() {
            for (ty, index) in tys.iter_mut() {
                ty.apply(self);

                if let Some(index) = index {
                    self.refine_index(index);
                }
            }

            *tys = Vec::from_iter(tys.drain(..).fold(
                HashMap::<Ty, Option<usize>>::new(),
                |mut tys, (ty, index)| {
                    let entry = tys.entry(ty).or_default();
                    if let Some(index) = index {
                        entry.get_or_insert(index);
                    }

                    tys
                },
            ));
        }

        // TODO: Set to `true` when a specific change is made
        let progress = self.constraints.tys != results;

        // TODO: If no progress, apply bounds; still no progress, apply
        // defaults; etc.

        let result = progress.then(|| results.clone());
        self.constraints.tys = results;

        result
    }
}

impl Ty {
    fn unify_with(&mut self, other: &Ty, session: &mut Session, success: &mut bool) {
        self.apply(session);

        match (self, other) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => {}
            (_, Ty::Of(..)) | (Ty::Of(..), _) => {
                unreachable!("`Ty::Of` should be replaced with `Ty::Group`")
            }
            (_, Ty::Instantiate(..)) | (Ty::Instantiate(..), _) => {
                // Skip; these are treated as opaque until instantiated
            }
            (ty @ &mut Ty::Group(index), other) => {
                let mut other = other.clone();
                other.apply(session);
                session.groups[index].ty = Some(other.clone());
                *ty = other;
                ty.apply(session);
            }
            (other, &Ty::Group(index)) => match session.groups[index].ty.clone() {
                Some(group_ty) => {
                    other.unify_with(&group_ty, session, success);
                }
                None => {
                    other.apply(session);
                    session.groups[index].ty = Some(other.clone());
                }
            },
            (
                Ty::Named { name, parameters },
                Ty::Named {
                    name: other_name,
                    parameters: other_parameters,
                },
            ) if name == other_name => {
                for (parameter, other) in parameters.iter_mut().zip(other_parameters) {
                    parameter.unify_with(other, session, success);
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
                    input.unify_with(other, session, success);
                }

                output.unify_with(other_output, session, success);
            }
            (
                Ty::Tuple { elements },
                Ty::Tuple {
                    elements: other_elements,
                },
            ) if elements.len() == other_elements.len() => {
                for (element, other) in elements.iter_mut().zip(other_elements) {
                    element.unify_with(other, session, success);
                }
            }
            _ => *success = false,
        }
    }

    fn apply(&mut self, session: &mut Session) {
        self.apply_inner(session, &mut Vec::new());
    }

    fn apply_inner(&mut self, session: &mut Session, stack: &mut Vec<usize>) {
        self.traverse_mut(&mut |ty| {
            if let Ty::Group(index) = *ty {
                if stack.contains(&index) {
                    return;
                }

                stack.push(index);

                if let Some(group_ty) = &session.groups[index].ty {
                    *ty = group_ty.clone();
                    ty.apply_inner(session, stack);
                }

                stack.pop();
            }
        });
    }
}
