use crate::constraints::{Constraints, Ty, TyConstraints};
use ena::unify::InPlaceUnificationTable;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone, Default)]
pub struct TyGroups {
    indices: BTreeMap<NodeId, usize>,
    tys: Vec<Vec<Ty>>,
}

impl TyGroups {
    pub fn insert_group(&mut self, ty: Ty) -> usize {
        let index = self.tys.len();
        self.tys.push(vec![ty]);
        index
    }

    pub fn assign_node_to_index(&mut self, node: NodeId, index: usize) {
        self.indices.insert(node, index);
    }

    pub fn index_of(&self, node: NodeId) -> Option<usize> {
        self.indices.get(&node).copied()
    }

    pub fn tys_at(&self, index: usize) -> &[Ty] {
        &self.tys[index]
    }

    pub fn tys_at_mut(&mut self, index: usize) -> &mut Vec<Ty> {
        &mut self.tys[index]
    }

    pub fn nodes_in_group(&self, index: usize) -> impl Iterator<Item = NodeId> {
        self.indices
            .iter()
            .filter_map(move |(&node, &i)| (i == index).then_some(node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.indices.keys().copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = (usize, &[Ty])> {
        self.tys
            .iter()
            .enumerate()
            .map(|(index, tys)| (index, tys.as_slice()))
    }
}

#[derive(Debug, Default)]
pub struct Typechecker {
    keys: GroupKeys,
    unify: InPlaceUnificationTable<GroupKey>,
    groups: BTreeMap<GroupKey, Ty>,    // types that unified
    others: BTreeMap<NodeId, Vec<Ty>>, // types that failed to unify
    progress: Progress,
}

impl Typechecker {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run_with(self, constraints: Constraints) -> TyGroups {
        self.run_with_until(constraints, None)
            .unwrap_or_else(|_| unreachable!())
    }

    pub fn run_with_until(
        mut self,
        mut constraints: Constraints,
        limit: impl Into<Option<usize>>,
    ) -> Result<TyGroups, Self> {
        let limit = limit.into();

        let mut counter = 0;
        while limit.is_none_or(|limit| counter < limit) {
            counter += 1;

            // TODO: If no progress, apply bounds; still no progress, apply
            // defaults; etc., using `and_then(..)`
            match self.add_tys(mem::take(&mut constraints.tys)) {
                Progress::Progressed => continue,
                Progress::NoProgress => return Ok(self.to_ty_groups(constraints.nodes)),
            }
        }

        // Reached limit
        Err(self)
    }

    fn add_tys(&mut self, constraints: TyConstraints) -> Progress {
        // Form better groups by first adding constraints that reference other
        // nodes directly
        let (of, others): (Vec<_>, Vec<_>) = constraints
            .into_iter()
            .flat_map(|(node, tys)| tys.into_iter().map(move |ty| (node, ty)))
            .partition(|(_, ty)| matches!(ty, Ty::Of(_)));

        let (incomplete, complete): (Vec<_>, Vec<_>) =
            others.into_iter().partition(|(_, ty)| ty.is_incomplete());

        for (node, mut ty) in [of, incomplete, complete].into_iter().flatten() {
            if self.unify(&mut Ty::Of(node), &mut ty).is_err() {
                // No need to generate a diagnostic here; feedback is
                // generated whenever an expression has multiple types
                self.others.entry(node).or_default().push(ty);
            }
        }

        self.progress.take()
    }

    fn to_ty_groups(&self, nodes: impl IntoIterator<Item = NodeId>) -> TyGroups {
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
        for node in nodes {
            if ty_groups.index_of(node).is_none() {
                let index = ty_groups.insert_group(Ty::Unknown);
                ty_groups.assign_node_to_index(node, index);
            }
        }

        ty_groups
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

                if let Some(mut representative) = self.groups.remove(&representative_key) {
                    self.apply(&mut representative);

                    self.groups
                        .insert(representative_key, representative.clone());

                    *ty = representative;
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

#[derive(Debug, Default)]
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
enum Progress {
    #[default]
    NoProgress,
    Progressed,
}

impl Progress {
    fn and_then(self, f: impl FnOnce() -> Self) -> Self {
        use Progress::*;

        match self {
            NoProgress => NoProgress,
            Progressed => f(),
        }
    }

    fn set(&mut self) {
        *self = Progress::Progressed;
    }

    fn take(&mut self) -> Self {
        mem::take(self)
    }
}
