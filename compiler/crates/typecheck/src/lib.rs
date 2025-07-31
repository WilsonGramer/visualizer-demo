pub mod constraints;
pub mod debug;
pub mod feedback;

use constraints::{Bound, Constraint, Instantiation, Substitutions, Ty};
use ena::unify::InPlaceUnificationTable;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    mem,
    rc::Rc,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone, Default)]
pub struct TyGroups {
    indices: BTreeMap<NodeId, u32>,
    tys: Vec<Vec<Ty>>,
}

impl TyGroups {
    fn insert_group(&mut self, ty: Ty) -> u32 {
        let index = self.tys.len() as u32;
        self.tys.push(vec![ty]);
        index
    }

    fn assign_node_to_index(&mut self, node: NodeId, index: u32) {
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

#[derive(Debug, Clone)]
pub struct Instance {
    pub id: NodeId,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<Constraint>,
}

// TODO: Make this a trait
#[derive(Clone)]
pub struct TypeProvider<'a> {
    copy_node: Rc<RefCell<dyn FnMut(NodeId) -> NodeId + 'a>>,
    get_trait_instances:
        Rc<RefCell<dyn FnMut(NodeId) -> Vec<(NodeId, BTreeMap<NodeId, NodeId>)> + 'a>>,
    flag_resolved: Rc<RefCell<dyn FnMut(NodeId, Bound, NodeId) + 'a>>,
    flag_unresolved: Rc<RefCell<dyn FnMut(NodeId, Bound) + 'a>>,
}

impl<'a> TypeProvider<'a> {
    pub fn new(
        copy_node: impl FnMut(NodeId) -> NodeId + 'a,
        get_trait_instances: impl FnMut(NodeId) -> Vec<(NodeId, BTreeMap<NodeId, NodeId>)> + 'a,
        flag_resolved: impl FnMut(NodeId, Bound, NodeId) + 'a,
        flag_unresolved: impl FnMut(NodeId, Bound) + 'a,
    ) -> Self {
        TypeProvider {
            copy_node: Rc::new(RefCell::new(copy_node)),
            get_trait_instances: Rc::new(RefCell::new(get_trait_instances)),
            flag_resolved: Rc::new(RefCell::new(flag_resolved)),
            flag_unresolved: Rc::new(RefCell::new(flag_unresolved)),
        }
    }
}

impl Debug for TypeProvider<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypeProvider").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct Typechecker<'a> {
    provider: TypeProvider<'a>,
    nodes: BTreeSet<NodeId>,
    keys: GroupKeys,
    /// Tracks unified groups
    unify: InPlaceUnificationTable<GroupKey>,
    /// Types that unified
    groups: BTreeMap<GroupKey, Ty>,
    /// Types that failed to unify
    others: BTreeMap<NodeId, Vec<Ty>>,
    queue: Vec<Constraint>,
    progress: Progress,
    error: bool,
}

impl<'a> Typechecker<'a> {
    pub fn with_provider(provider: TypeProvider<'a>) -> Self {
        Typechecker {
            provider,
            nodes: Default::default(),
            keys: Default::default(),
            unify: Default::default(),
            groups: Default::default(),
            others: Default::default(),
            queue: Default::default(),
            progress: Default::default(),
            error: false,
        }
    }

    pub fn insert_nodes(&mut self, nodes: impl IntoIterator<Item = NodeId>) {
        self.nodes.extend(nodes);
    }

    pub fn insert_constraints(&mut self, constraints: impl IntoIterator<Item = Constraint>) {
        self.queue.extend(constraints);
        self.run();
    }

    pub fn to_ty_groups(&self) -> TyGroups {
        let mut ty_groups = TyGroups::default();

        let mut unify = self.unify.clone();

        for (representative_key, mut ty) in self.groups.clone().into_iter() {
            self.try_apply_ty(&mut ty, &mut unify);

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

        ty_groups
    }
}

impl Typechecker<'_> {
    fn run(&mut self) {
        loop {
            let progress = self
                .run_tys()
                .or_else(|| self.run_instantiations())
                .or_else(|| self.run_bounds());
            // TODO: run_defaults(), etc.

            match progress {
                Progress::Progressed => continue,
                Progress::NoProgress => break,
            }
        }

        assert!(self.queue.is_empty());
    }

    fn run_tys(&mut self) -> Progress {
        let mut tys = Vec::new();
        self.queue = mem::take(&mut self.queue)
            .into_iter()
            .filter_map(|constraint| match constraint {
                Constraint::Ty(node, ty) => {
                    tys.push((node, ty));
                    None
                }
                _ => Some(constraint),
            })
            .collect();

        // Form better groups by first processing constraints that reference
        // other nodes directly, followed by other incomplete types
        tys.sort_by_key(|(_, ty)| match ty {
            Ty::Of(_) => 0,
            ty if ty.is_incomplete() => 1,
            _ => 2,
        });

        for (node, mut ty) in tys {
            // `Ty::Of(node)` will resolve to the representative for `node`, so
            // this effectively unifies the node's type with the other types in
            // the node's group
            let result = self.unify_tys(&mut Ty::Of(node), &mut ty);

            if result.is_err() {
                self.error = true;
                self.others.entry(node).or_default().push(ty);
            }
        }

        self.progress.take()
    }

    fn run_instantiations(&mut self) -> Progress {
        let mut instantiations = Vec::new();
        self.queue = mem::take(&mut self.queue)
            .into_iter()
            .filter_map(|constraint| match constraint {
                Constraint::Instantiation(instantiation) => {
                    instantiations.push(instantiation);
                    None
                }
                _ => Some(constraint),
            })
            .collect();

        for instantiation in instantiations {
            let constraints = self.apply_instantiation(instantiation);
            self.queue.extend(constraints);
            self.progress.set();
        }

        self.progress.take()
    }

    fn run_bounds(&mut self) -> Progress {
        let mut bounds = Vec::new();
        self.queue = mem::take(&mut self.queue)
            .into_iter()
            .filter_map(|constraint| match constraint {
                Constraint::Bound(bound) => {
                    bounds.push(bound);
                    None
                }
                _ => Some(constraint),
            })
            .collect();

        for bound in bounds {
            // Use a temporary node for the bound while resolving.
            let temp_node = (self.provider.copy_node.borrow_mut())(bound.node);

            // Get the current type of the node, if there is one, without
            // actually unifying it with anything.
            let mut bound_ty = Ty::Of(bound.node);
            self.try_apply_ty(&mut bound_ty, &mut self.unify.clone());
            if bound_ty != Ty::Of(bound.node) {
                self.insert_constraints([Constraint::Ty(temp_node, bound_ty.clone())])
            }

            // Instantiate the bound with the trait's type.
            self.insert_constraints([Constraint::Instantiation(Instantiation {
                constraints: vec![Constraint::Ty(temp_node, Ty::Of(bound.tr))],
                substitutions: bound.substitutions.clone(),
            })]);

            let instances = self.provider.get_trait_instances.borrow_mut()(bound.tr);

            let mut candidates = Vec::new();
            for (instance, parameters) in instances {
                // Apply the instance's constraints to a copy of the
                // typechecker, so if the instance fails to match, we can reset
                let mut copy = self.clone();
                copy.error = false;

                // To resolve an instance, we need to satisfy the bound's
                // substitutions, the instance's substitutions, and the node's
                // type. All three are applied to the temporary node, so they
                // will unify.
                copy.queue = vec![Constraint::Instantiation(Instantiation {
                    constraints: vec![Constraint::Ty(temp_node, Ty::Of(instance))],
                    substitutions: Substitutions::from(parameters),
                })];

                // Recursive bounds will be resolved here.
                copy.run();

                if copy.error {
                    // Bound and instance constraints didn't unify; try the next
                    // candidate.
                    continue;
                }

                candidates.push((instance, copy));
            }

            if candidates.len() != 1 {
                self.provider.flag_unresolved.borrow_mut()(temp_node, bound);

                continue;
            }

            let (instance, copy) = candidates.into_iter().next().unwrap();

            // Incorporate the resolved types from the selected instance
            *self = copy;

            self.progress.set();
            self.provider.flag_resolved.borrow_mut()(temp_node, bound, instance);
        }

        self.progress.take()
    }
}

impl Typechecker<'_> {
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

impl Typechecker<'_> {
    fn try_apply_ty(&self, ty: &mut Ty, unify: &mut InPlaceUnificationTable<GroupKey>) {
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = *ty {
                let Some(key) = self.try_key_for_node(node) else {
                    return;
                };

                let representative = unify.find(key);

                if let Some(representative) = self.groups.get(&representative) {
                    *ty = representative.clone();
                    self.try_apply_ty(ty, unify);
                }
            }
        });
    }

    fn apply_ty(&mut self, ty: &mut Ty) {
        ty.traverse_mut(&mut |ty| {
            if let Ty::Of(node) = *ty {
                let key = self.key_for_node(node);
                let representative_key = self.unify.find(key);
                let representative = self.node_for_key(representative_key);
                if let Some(mut representative_ty) = self.groups.remove(&representative_key) {
                    self.apply_ty(&mut representative_ty);

                    self.groups
                        .insert(representative_key, representative_ty.clone());

                    *ty = representative_ty;
                } else {
                    *ty = Ty::Of(representative);
                }
            }
        });
    }

    fn apply_instantiation(&mut self, mut instantiation: Instantiation) -> Vec<Constraint> {
        let mut unify = self.unify.clone();

        instantiation
            .constraints
            .into_iter()
            .map(|mut constraint| {
                match &mut constraint {
                    Constraint::Ty(_, ty) => {
                        self.instantiate_ty(ty, &mut instantiation.substitutions, &mut unify);
                    }
                    Constraint::Instantiation(..) => {
                        panic!("cannot have nested instantiations")
                    }
                    Constraint::Bound(bound) => {
                        for ty in bound.substitutions.0.values_mut() {
                            self.instantiate_ty(ty, &mut instantiation.substitutions, &mut unify);
                        }
                    }
                }

                constraint
            })
            .collect()
    }

    fn instantiate_ty(
        &mut self,
        ty: &mut Ty,
        substitutions: &mut Substitutions,
        unify: &mut InPlaceUnificationTable<GroupKey>,
    ) {
        self.try_apply_ty(ty, unify);

        ty.traverse_mut(&mut |ty| {
            if let Ty::Parameter(parameter) = ty {
                if let Some(substitution) = substitutions.0.get(parameter).cloned() {
                    *ty = substitution;
                } else {
                    let copy = (self.provider.copy_node.borrow_mut())(*parameter);
                    self.nodes.insert(copy);
                    substitutions.0.insert(*parameter, Ty::Of(copy));
                    *ty = Ty::Of(copy);
                }
            }
        });
    }

    fn unify_tys(&mut self, left: &mut Ty, right: &mut Ty) -> Result<(), ()> {
        self.apply_ty(left);
        self.apply_ty(right);

        match (&mut *left, &mut *right) {
            (Ty::Parameter(left), Ty::Parameter(right)) => {
                if left != right {
                    return Err(());
                }
            }
            (Ty::Of(left_node), Ty::Of(right_node)) => {
                self.unify_nodes(left_node, right_node);
            }
            (other, ty @ &mut Ty::Of(node)) | (ty @ &mut Ty::Of(node), other) => {
                let key = self.key_for_node(node);
                if let Some(mut existing) = self.groups.insert(key, other.clone()) {
                    self.unify_tys(ty, &mut existing)?;
                }

                *ty = other.clone();

                self.progress.set();
            }
            (
                Ty::Named {
                    name: left_name,
                    parameters: left_substitutions,
                },
                Ty::Named {
                    name: right_name,
                    parameters: right_substitutions,
                },
            ) if left_name == right_name
                && left_substitutions.len() == right_substitutions.len() =>
            {
                for (left, right) in left_substitutions
                    .values_mut()
                    .zip(right_substitutions.values_mut())
                {
                    self.unify_tys(left, right)?;
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
                    self.unify_tys(left, right)?;
                }

                self.unify_tys(left_output, right_output)?;
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
                    self.unify_tys(left, right)?;
                }
            }
            _ => return Err(()),
        }

        Ok(())
    }

    fn unify_nodes(&mut self, left_node: &mut NodeId, right_node: &mut NodeId) {
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
    fn set(&mut self) {
        *self = Progress::Progressed;
    }

    fn take(&mut self) -> Self {
        mem::take(self)
    }

    fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Progress::Progressed => Progress::Progressed,
            Progress::NoProgress => f(),
        }
    }
}
