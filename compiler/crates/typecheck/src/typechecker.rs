use crate::{
    constraints::{Bound, BoundConstraints, Constraint, Ty, TyConstraints},
    id::TypedNodeId,
};
use ena::unify::InPlaceUnificationTable;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    mem,
    rc::Rc,
};
use wipple_compiler_trace::{NodeId, Rule};

pub static UNRESOLVED_TRAIT: Rule = Rule::new("unresolved trait");

#[derive(Debug, Clone, Default)]
pub struct TyGroups {
    indices: BTreeMap<TypedNodeId, u32>,
    tys: Vec<Vec<Ty>>,
}

impl TyGroups {
    fn insert_group(&mut self, ty: Ty) -> u32 {
        let index = self.tys.len() as u32;
        self.tys.push(vec![ty]);
        index
    }

    fn assign_node_to_index(&mut self, node: TypedNodeId, index: u32) {
        self.indices.insert(node, index);
    }

    pub fn index_of(&self, node: TypedNodeId) -> Option<u32> {
        self.indices.get(&node).copied()
    }

    pub fn tys_at(&self, index: u32) -> &[Ty] {
        &self.tys[index as usize]
    }

    pub fn tys_at_mut(&mut self, index: u32) -> &mut Vec<Ty> {
        &mut self.tys[index as usize]
    }

    pub fn nodes_in_group(&self, index: u32) -> impl Iterator<Item = TypedNodeId> {
        self.indices
            .iter()
            .filter_map(move |(&node, &i)| (i == index).then_some(node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = TypedNodeId> {
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

#[derive(Clone)]
pub struct TypeProvider<'a> {
    get_trait_instances:
        Rc<RefCell<dyn FnMut(NodeId) -> Vec<(NodeId, BTreeMap<NodeId, NodeId>, Rule)> + 'a>>,
    flag_resolved: Rc<RefCell<dyn FnMut(TypedNodeId, Bound, NodeId) + 'a>>,
    flag_unresolved: Rc<RefCell<dyn FnMut(TypedNodeId, Bound) + 'a>>,
}

impl<'a> TypeProvider<'a> {
    pub fn new(
        get_trait_instances: impl FnMut(NodeId) -> Vec<(NodeId, BTreeMap<NodeId, NodeId>, Rule)> + 'a,
        flag_resolved: impl FnMut(TypedNodeId, Bound, NodeId) + 'a,
        flag_unresolved: impl FnMut(TypedNodeId, Bound) + 'a,
    ) -> Self {
        TypeProvider {
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
    unify: InPlaceUnificationTable<GroupKey>,
    groups: BTreeMap<GroupKey, Ty>,         // types that unified
    others: BTreeMap<TypedNodeId, Vec<Ty>>, // types that failed to unify
    queue: Vec<Constraint>,                 // ordered
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

    pub fn insert_tys(&mut self, constraints: &TyConstraints) {
        // Instantiations must come last (since they need their definition's
        // type). Otherwise, form better groups by first adding constraints that
        // reference other nodes directly, followed by other incomplete types
        // (FIXME: Make instantiations their own type of constraint)
        let mut of = Vec::new();
        let mut incomplete = Vec::new();
        let mut complete = Vec::new();
        let mut instantiate = Vec::new();
        for (&node, tys) in constraints {
            for (ty, rule) in tys.iter().cloned() {
                if let Ty::Of(_) = ty {
                    of.push((node, (ty.clone(), rule)));
                } else if let Ty::Instantiate { .. } = ty {
                    instantiate.push((node, (ty.clone(), rule)));
                } else if ty.is_incomplete() {
                    incomplete.push((node, (ty.clone(), rule)));
                } else {
                    complete.push((node, (ty.clone(), rule)));
                }
            }
        }

        for (node, (ty, rule)) in [of, incomplete, complete, instantiate]
            .into_iter()
            .flatten()
        {
            self.queue.push(Constraint::Ty(node, ty, rule));
        }

        self.run();
    }

    pub fn insert_bounds(&mut self, bounds: &BoundConstraints) {
        for (node, bounds) in bounds {
            for (bound, rule) in bounds {
                self.queue
                    .push(Constraint::Bound(*node, bound.clone(), *rule));
            }
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
            let node = TypedNodeId::from(node);

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
            // FIXME: Run all stages in order each time instead of `or_else`?
            let progress = self.run_tys().or_else(|| self.run_bounds());
            // TODO: `.or_else(|| self.run_defaults())`, etc.

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
            .filter_map(|constraint| match constraint {
                Constraint::Ty(node, ty, _) => {
                    tys.push((node, ty));
                    None
                }
                _ => Some(constraint),
            })
            .collect();

        for (node, mut ty) in tys {
            // `Ty::Of(node)` will resolve to the representative for `node`, so
            // this effectively unifies the node's type with the other types in
            // the node's group
            let result = self.unify(&mut Ty::Of(node), &mut ty);

            if result.is_err() {
                self.error = true;
                self.others.entry(node).or_default().push(ty);
            }
        }

        self.progress.take()
    }

    fn run_bounds(&mut self) -> Progress {
        let mut bounds = Vec::new();
        self.queue = mem::take(&mut self.queue)
            .into_iter()
            .filter_map(|constraint| match constraint {
                Constraint::Bound(node, bound, rule) => {
                    bounds.push((node, bound, rule));
                    None
                }
                _ => Some(constraint),
            })
            .collect();

        for (node, bound, rule) in bounds {
            // To resolve an instance, we need to satisfy the intersection of
            // the bound's substitutions (1) and an instance's substitutions (2).

            let instances = self.provider.get_trait_instances.borrow_mut()(bound.tr);

            let node = TypedNodeId::instantiate(node, bound.instantiation);

            // (1)
            self.insert_tys(&TyConstraints::from([(node, vec![(bound.as_ty(), rule)])]));

            let mut candidates = Vec::new();
            for (instance, substitutions, rule) in instances {
                let instance_bound = Bound {
                    tr: bound.tr,
                    instantiation: bound.instantiation,
                    substitutions: Some(substitutions),
                };

                // Apply the instance's constraints to a copy of the
                // typechecker, so if the instance fails to match, we can reset
                let mut copy = self.clone();
                copy.error = false;

                // (2)
                copy.insert_tys(&TyConstraints::from([(
                    node,
                    vec![(instance_bound.as_ty(), rule)],
                )]));

                // (NOTE: `insert_tys` calls `run` and will also resolved
                // nested bounds)

                let error = copy.others.contains_key(&node);
                if !error {
                    candidates.push((instance, copy));
                }
            }

            if candidates.len() != 1 {
                self.provider.flag_unresolved.borrow_mut()(node, bound);

                continue;
            }

            let (instance, copy) = candidates.into_iter().next().unwrap();

            // Incorporate the resolved types from the selected instance
            *self = copy;

            self.progress.set();
            self.provider.flag_resolved.borrow_mut()(node, bound, instance);
        }

        self.progress.take()
    }
}

impl Typechecker<'_> {
    fn key_for_node(&mut self, node: TypedNodeId) -> GroupKey {
        self.keys
            .key_for_node(node, || self.unify.new_key(Group::new(node)))
    }

    fn try_key_for_node(&self, node: TypedNodeId) -> Option<GroupKey> {
        self.keys.try_key_for_node(node)
    }

    fn node_for_key(&self, key: GroupKey) -> TypedNodeId {
        self.keys.node_for_key(key)
    }
}

impl Typechecker<'_> {
    fn try_apply(&self, ty: &mut Ty, unify: &mut InPlaceUnificationTable<GroupKey>) {
        ty.traverse_mut(&mut |ty| match *ty {
            Ty::Of(node) => {
                let Some(key) = self.try_key_for_node(node) else {
                    return;
                };

                let representative = unify.find(key);

                if let Some(representative) = self.groups.get(&representative) {
                    *ty = representative.clone();
                    self.try_apply(ty, unify);
                }
            }
            Ty::Instantiate { .. } => panic!("uninstantiated type"),
            _ => {}
        });
    }

    fn apply(&mut self, ty: &mut Ty) {
        ty.traverse_mut(&mut |ty| match ty.clone() {
            Ty::Of(node) => {
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
            Ty::Instantiate {
                instantiation,
                definition,
                substitutions,
            } => {
                let mut unify = self.unify.clone();

                // Deeply resolve the definition's type
                *ty = Ty::of(definition);
                self.try_apply(ty, &mut unify);

                loop {
                    let mut progress = false;
                    ty.traverse_mut(&mut |ty| {
                        match *ty {
                            Ty::Parameter(parameter) => {
                                // If `substitutions` is `None`, substitute all
                                // parameters. Otherwise, only substitute the
                                // parameters provided
                                if let Some(substitutions) = &substitutions {
                                    let Some(substitution) = substitutions.get(&parameter).copied()
                                    else {
                                        return;
                                    };

                                    // Deeply resolve the substitution's type
                                    let mut substitution = Ty::of(substitution);
                                    self.try_apply(&mut substitution, &mut unify);

                                    if let Ty::Of(substitution) = substitution {
                                        *ty =
                                            Ty::Of(substitution.instantiated_under(instantiation));
                                    } else {
                                        // If the substitution resolved to an actual
                                        // type, use that instead of the instantiated
                                        // node ID
                                        *ty = substitution;
                                    }
                                } else {
                                    *ty =
                                        Ty::Of(TypedNodeId::instantiate(parameter, instantiation));
                                }

                                progress = true;
                            }
                            Ty::Instantiated(parameter) => {
                                // The parameter is being referenced from a
                                // bound or another constraint that isn't the
                                // type signature. That means the parameter has
                                // already been substituted above, so just refer
                                // to that existing substitution
                                *ty = Ty::Of(TypedNodeId::instantiate(parameter, instantiation));
                                progress = true;
                            }
                            _ => {}
                        }
                    });

                    if !progress {
                        break;
                    }
                }

                // Apply again to deeply resolve any substitutions
                self.try_apply(ty, &mut unify);
            }
            _ => {}
        });
    }

    fn unify(&mut self, left: &mut Ty, right: &mut Ty) -> Result<(), ()> {
        self.apply(left);
        self.apply(right);

        match (&mut *left, &mut *right) {
            (Ty::Instantiate { .. }, _) | (_, Ty::Instantiate { .. }) => {
                panic!("uninstantiated type")
            }
            (Ty::Parameter(left_parameter), Ty::Parameter(right_parameter)) => {
                if *left_parameter == *right_parameter {}
            }
            (Ty::Of(left_node), Ty::Of(right_node)) => {
                self.unify_nodes(left_node, right_node);
            }
            (other, ty @ &mut Ty::Of(node)) | (ty @ &mut Ty::Of(node), other) => {
                let key = self.key_for_node(node);
                if let Some(mut existing) = self.groups.insert(key, other.clone()) {
                    self.unify(ty, &mut existing)?;
                }

                *ty = other.clone();

                self.progress.set();
            }
            (
                Ty::Named {
                    name: left_name,
                    substitutions: left_substitutions,
                },
                Ty::Named {
                    name: right_name,
                    substitutions: right_substitutions,
                },
            ) if left_name == right_name
                && left_substitutions.len() == right_substitutions.len() =>
            {
                for (left, right) in left_substitutions
                    .values_mut()
                    .zip(right_substitutions.values_mut())
                {
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

    fn unify_nodes(&mut self, left_node: &mut TypedNodeId, right_node: &mut TypedNodeId) {
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
struct Group(BTreeSet<TypedNodeId>);

impl Group {
    pub fn new(node: TypedNodeId) -> Self {
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
    keys: BTreeMap<TypedNodeId, GroupKey>,
    nodes: BTreeMap<GroupKey, TypedNodeId>,
}

impl GroupKeys {
    pub fn key_for_node(&mut self, node: TypedNodeId, init: impl FnOnce() -> GroupKey) -> GroupKey {
        *self.keys.entry(node).or_insert_with(|| {
            let key = init();
            self.nodes.insert(key, node);
            key
        })
    }

    fn try_key_for_node(&self, node: TypedNodeId) -> Option<GroupKey> {
        self.keys.get(&node).copied()
    }

    pub fn update_representative(&mut self, node: TypedNodeId, group: GroupKey) {
        self.keys.insert(node, group);
        self.nodes.insert(group, node);
    }

    pub fn node_for_key(&self, key: GroupKey) -> TypedNodeId {
        self.try_node_for_key(key).unwrap()
    }

    fn try_node_for_key(&self, key: GroupKey) -> Option<TypedNodeId> {
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
