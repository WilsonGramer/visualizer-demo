use crate::constraints::{Constraint, GenericConstraints, Ty, TyConstraints};
use ena::unify::InPlaceUnificationTable;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone, Default)]
pub struct TyGroups {
    indices: BTreeMap<NodeId, u32>,
    tys: Vec<Vec<Ty>>,
}

impl TyGroups {
    pub fn insert_group(&mut self, ty: Ty) -> u32 {
        let index = self.tys.len() as u32;
        self.tys.push(vec![ty]);
        index
    }

    pub fn assign_node_to_index(&mut self, node: NodeId, index: u32) {
        self.indices.insert(node, index);
    }

    pub fn index_of(&self, node: NodeId) -> Option<u32> {
        self.indices.get(&node).copied()
    }

    pub fn tys_at(&self, index: u32) -> &[Ty] {
        &self.tys[index as usize]
    }

    pub fn tys_at_mut(&mut self, index: u32) -> &mut Vec<Ty> {
        &mut self.tys[index as usize]
    }

    pub fn nodes_in_group(&self, index: u32) -> impl Iterator<Item = NodeId> {
        self.indices
            .iter()
            .filter_map(move |(&node, &i)| (i == index).then_some(node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.indices.keys().copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = (u32, &[Ty])> {
        self.tys
            .iter()
            .enumerate()
            .map(|(index, tys)| (index as u32, tys.as_slice()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Typechecker {
    nodes: BTreeSet<NodeId>,
    keys: GroupKeys,
    unify: InPlaceUnificationTable<GroupKey>,
    groups: BTreeMap<GroupKey, Ty>,    // types that unified
    others: BTreeMap<NodeId, Vec<Ty>>, // types that failed to unify
    queue: Vec<(NodeId, Constraint)>,  // ordered
    progress: Progress,
}

impl Typechecker {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_nodes(&mut self, nodes: impl IntoIterator<Item = NodeId>) {
        self.nodes.extend(nodes);
    }

    pub fn insert_tys(&mut self, constraints: &TyConstraints) {
        // Form better groups by first adding constraints that reference other
        // nodes directly, followed by other incomplete types
        let mut of = Vec::new();
        let mut incomplete = Vec::new();
        let mut complete = Vec::new();
        for (&node, tys) in constraints {
            for ty in tys {
                if let Ty::Of(_) = ty {
                    of.push((node, ty.clone()))
                } else if ty.is_incomplete() {
                    incomplete.push((node, ty.clone()))
                } else {
                    complete.push((node, ty.clone()))
                }
            }
        }

        for (node, ty) in [of, incomplete, complete].into_iter().flatten() {
            self.queue.push((node, Constraint::Ty(ty)));
        }

        self.run();
    }

    pub fn insert_generics(&mut self, constraints: &GenericConstraints) {
        // Make instantiated copies of the definition's constraints
        for &(node, definition) in constraints {
            // Resolve the definition's type on its own...
            let mut definition_typechecker = self.clone();
            let key = definition_typechecker.key_for_node(node);

            // ...and then make an instantiated copy to use in `self`
            let mut definition_ty = Ty::Of(definition);
            definition_typechecker.apply(&mut definition_ty);
            definition_ty.traverse_mut(&mut |ty| {
                if let Ty::Of(other) = ty {
                    let existing_namespace = other.namespace.replace(key.0);
                    assert!(existing_namespace.is_none());
                }
            });

            self.queue.push((node, Constraint::Ty(definition_ty)));
        }

        self.run();
    }

    pub fn to_ty_groups(&self) -> TyGroups {
        let mut ty_groups = TyGroups::default();

        let mut unify = self.unify.clone();

        for (representative_key, mut ty) in self.groups.clone().into_iter() {
            self.try_apply(&mut ty, &mut unify);

            let index = ty_groups.insert_group(ty);

            let nodes = unify.probe_value(representative_key).0;
            for node in nodes {
                ty_groups.assign_node_to_index(node, index);
            }
        }

        for (&node, others) in &self.others {
            for ty in others {
                match ty_groups.index_of(node) {
                    Some(index) => {
                        ty_groups.tys_at_mut(index).push(ty.clone());
                    }
                    None => {
                        let index = ty_groups.insert_group(ty.clone());
                        ty_groups.assign_node_to_index(node, index);
                    }
                }
            }
        }

        // Any remaining nodes have unknown types
        for &node in &self.nodes {
            if ty_groups.index_of(node).is_none() {
                let index = ty_groups.insert_group(Ty::Unknown);
                ty_groups.assign_node_to_index(node, index);
            }
        }

        // Remove namespaces from all node IDs
        ty_groups.indices.retain(|node, _| node.namespace.is_none());

        // Replace unresolved `Ty::Of` types with `Ty::Unknown`
        for ty in ty_groups.tys.iter_mut().flatten() {
            ty.traverse_mut(&mut |ty| {
                if let Ty::Of(_) = *ty {
                    *ty = Ty::Unknown;
                }
            });
        }

        ty_groups
    }
}

impl Typechecker {
    fn run(&mut self) {
        loop {
            // TODO: If no progress, apply bounds; still no progress, apply
            // defaults; etc., using `and_then(..)`
            let progress = self.run_tys();

            match progress {
                Progress::Progressed => continue,
                Progress::NoProgress => break,
            }
        }
    }

    fn run_tys(&mut self) -> Progress {
        let mut tys = Vec::new();
        self.queue = mem::take(&mut self.queue)
            .into_iter()
            .filter_map(|(node, constraint)| match constraint {
                Constraint::Ty(ty) => {
                    tys.push((node, ty));
                    None
                }
                _ => Some((node, constraint)),
            })
            .collect();

        for (node, mut ty) in tys {
            // `Ty::Of(node)` will resolve to the representative for `node`, so
            // this effectively unifies the node's type with the other types in
            // the node's group
            let result = self.unify(&mut Ty::Of(node), &mut ty);

            if result.is_err() {
                // No need to generate a diagnostic here; feedback is
                // generated whenever an expression has multiple types
                self.others.entry(node).or_default().push(ty);
            }
        }

        self.progress.take()
    }
}

impl Typechecker {
    fn key_for_node(&mut self, node: NodeId) -> GroupKey {
        self.keys
            .key_for_node(node, || self.unify.new_key(Group::new(node)))
    }

    fn try_key_for_node(&self, node: NodeId) -> Option<GroupKey> {
        self.keys.try_key_for_node(node)
    }

    fn node_for_key(&self, key: GroupKey) -> NodeId {
        self.keys.node_for_key(key)
    }
}

impl Typechecker {
    fn try_apply(&self, ty: &mut Ty, unify: &mut InPlaceUnificationTable<GroupKey>) {
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = *ty {
                let Some(key) = self.try_key_for_node(node) else {
                    *ty = Ty::Unknown;
                    return;
                };

                let representative = unify.find(key);

                if let Some(representative) = self.groups.get(&representative) {
                    *ty = representative.clone();
                    self.try_apply(ty, unify);
                }
            }
        });
    }

    fn apply(&mut self, ty: &mut Ty) {
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = *ty {
                let key = self.key_for_node(node);

                let representative_key = self.unify.find(key);
                let representative = self.node_for_key(representative_key);

                if let Some(mut representative_ty) = self.groups.remove(&representative_key) {
                    self.apply(&mut representative_ty);

                    self.groups
                        .insert(representative_key, representative_ty.clone());

                    *ty = representative_ty;
                } else {
                    *ty = Ty::Of(representative);
                }
            }
        });
    }

    fn unify(&mut self, left: &mut Ty, right: &mut Ty) -> Result<(), ()> {
        self.apply(left);
        self.apply(right);

        match (&mut *left, &mut *right) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => {}
            (Ty::Of(left_node), Ty::Of(right_node)) => {
                let left_key = self.key_for_node(*left_node);
                let right_key = self.key_for_node(*right_node);

                self.unify
                    .unify_var_var(left_key, right_key)
                    .unwrap_or_else(|e| match e {});

                let representative_key = self.unify.find(left_key);
                let representative = self.node_for_key(representative_key);

                // Move types from the old group to the new group
                for key in [left_key, right_key] {
                    if representative_key == key {
                        continue;
                    }

                    if let Some(ty) = self.groups.remove(&key) {
                        self.groups.insert(representative_key, ty);
                    }

                    let node = self.node_for_key(key);
                    self.keys.update_representative(node, representative_key);
                }

                *left_node = representative;
                *right_node = representative;

                self.progress.set();
            }
            (other, ty @ &mut Ty::Of(node)) | (ty @ &mut Ty::Of(node), other) => {
                let key = self.key_for_node(node);
                let existing = self.groups.insert(key, other.clone());
                assert!(existing.is_none());

                *ty = other.clone();

                self.progress.set();
            }
            (
                Ty::Named {
                    name: left_name,
                    parameters: left_parameters,
                },
                Ty::Named {
                    name: right_name,
                    parameters: right_parameters,
                },
            ) if left_name == right_name && left_parameters.len() == right_parameters.len() => {
                for (left, right) in left_parameters.iter_mut().zip(right_parameters) {
                    self.unify(left, right)?;
                }
            }
            (
                Ty::Function {
                    inputs: left_inputs,
                    output: left_output,
                },
                Ty::Function {
                    inputs: right_inputs,
                    output: right_output,
                },
            ) if left_inputs.len() == right_inputs.len() => {
                for (left, right) in left_inputs.iter_mut().zip(right_inputs) {
                    self.unify(left, right)?;
                }

                self.unify(left_output, right_output)?;
            }
            (
                Ty::Tuple {
                    elements: left_elements,
                },
                Ty::Tuple {
                    elements: right_elements,
                },
            ) if left_elements.len() == right_elements.len() => {
                for (left, right) in left_elements.iter_mut().zip(right_elements) {
                    self.unify(left, right)?;
                }
            }
            _ => return Err(()),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GroupKey(u32);

#[derive(Debug, Clone)]
struct Group(BTreeSet<NodeId>);

impl Group {
    pub fn new(node: NodeId) -> Self {
        Group(BTreeSet::from([node]))
    }
}

impl ena::unify::UnifyKey for GroupKey {
    type Value = Group;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(index: u32) -> Self {
        GroupKey(index)
    }

    fn tag() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl ena::unify::UnifyValue for Group {
    type Error = std::convert::Infallible;

    fn unify_values(left: &Self, right: &Self) -> Result<Self, Self::Error> {
        Ok(Group(left.0.union(&right.0).copied().collect()))
    }
}

#[derive(Debug, Clone, Default)]
struct GroupKeys {
    keys: BTreeMap<NodeId, GroupKey>,
    nodes: BTreeMap<GroupKey, NodeId>,
}

impl GroupKeys {
    pub fn key_for_node(&mut self, node: NodeId, init: impl FnOnce() -> GroupKey) -> GroupKey {
        *self.keys.entry(node).or_insert_with(|| {
            let key = init();
            self.nodes.insert(key, node);
            key
        })
    }

    fn try_key_for_node(&self, node: NodeId) -> Option<GroupKey> {
        self.keys.get(&node).copied()
    }

    pub fn update_representative(&mut self, node: NodeId, group: GroupKey) {
        self.keys.insert(node, group);
        self.nodes.insert(group, node);
    }

    pub fn node_for_key(&self, key: GroupKey) -> NodeId {
        self.try_node_for_key(key).unwrap()
    }

    fn try_node_for_key(&self, key: GroupKey) -> Option<NodeId> {
        self.nodes.get(&key).copied()
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
enum Progress {
    #[default]
    NoProgress,
    Progressed,
}

impl Progress {
    fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        use Progress::*;

        match self {
            NoProgress => f(),
            Progressed => Progressed,
        }
    }

    fn set(&mut self) {
        *self = Progress::Progressed;
    }

    fn take(&mut self) -> Self {
        mem::take(self)
    }
}
