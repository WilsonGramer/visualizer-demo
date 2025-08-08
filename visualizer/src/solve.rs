use crate::{Constraint, Group, GroupKey, GroupKeys, Instantiation, Substitutions, Ty, TyGroups};
use derive_where::derive_where;
use ena::unify::InPlaceUnificationTable;
use itertools::Itertools;
use std::{cell::RefCell, collections::BTreeMap, fmt::Debug, mem, rc::Rc};

#[derive_where(Debug, Clone)]
pub struct Instance<Db: crate::Db> {
    pub id: Db::Node,
    pub parameters: Vec<Db::Node>,
    pub constraints: Vec<Constraint<Db>>,
}

#[derive_where(Clone)]
pub struct Solver<'a, Db: crate::Db> {
    db: Rc<RefCell<&'a mut Db>>,
    keys: GroupKeys<Db>,
    unify: InPlaceUnificationTable<GroupKey<Db>>,
    groups: BTreeMap<GroupKey<Db>, Ty<Db>>,
    others: BTreeMap<Db::Node, Vec<Ty<Db>>>, // failed to unify
    queue: Vec<Constraint<Db>>,
    progress: Progress,
    error: bool,
}

impl<'a, Db: crate::Db> Solver<'a, Db> {
    pub fn new(db: &'a mut Db) -> Self {
        Solver {
            db: Rc::new(RefCell::new(db)),
            keys: Default::default(),
            unify: Default::default(),
            groups: Default::default(),
            others: Default::default(),
            queue: Default::default(),
            progress: Default::default(),
            error: false,
        }
    }

    pub fn insert(&mut self, constraints: impl IntoIterator<Item = Constraint<Db>>) {
        self.queue.extend(constraints);
        self.run();
    }

    pub fn finish(&self) -> TyGroups<Db> {
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

        let mut db = self.db.borrow_mut();

        let typed_nodes = db.typed_nodes().collect::<Vec<_>>();

        for node in typed_nodes {
            // Give untyped expressions a default type of `Unknown`
            if ty_groups.index_of(node).is_none() {
                let group_index = ty_groups.insert_group(Ty::Unknown(node));
                ty_groups.assign_node_to_index(node, group_index);
            }

            let tys = ty_groups
                .index_of(node)
                .map(|index| ty_groups.tys_at(index))
                .unwrap();

            let mut all_incomplete = true;
            for ty in tys {
                all_incomplete &= ty.is_incomplete();
                db.flag_type(node, ty.clone());
            }

            if all_incomplete {
                db.flag_incomplete_type(node);
            }
        }

        ty_groups
    }
}

impl<Db: crate::Db> Solver<'_, Db> {
    fn run(&mut self) {
        loop {
            let progress = self
                .run_tys()
                .or_else(|| self.run_instantiations())
                .or_else(|| self.run_bounds())
                // TODO: run_defaults(), etc.
                .or_else(|| self.fill_with_unknown());

            match progress {
                Progress::Progressed => continue,
                Progress::NoProgress => break,
            }
        }

        assert!(
            self.queue.is_empty(),
            "constraints remaining in the queue: {:#?}",
            self.queue
        );
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
            if !constraints.is_empty() {
                self.queue.extend(constraints);
                self.progress.set();
            }
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
            let temp_node = self.db.borrow_mut().clone_node(bound.node);

            // Instantiate the bound with the trait's type.
            self.insert([Constraint::Instantiation(Instantiation {
                constraints: vec![Constraint::Ty(temp_node, Ty::Of(bound.tr))],
                substitutions: bound.substitutions.clone(),
            })]);

            let instances = self.db.borrow_mut().get_trait_instances(bound.tr);

            let mut candidates = Vec::new();
            for (instance, substitutions) in instances {
                // Apply the instance's constraints to a copy of the
                // typechecker, so if the instance fails to match, we can reset
                let mut copy = self.clone();
                copy.error = false;

                // To resolve an instance, we need to satisfy the bound's
                // substitutions, the instance's substitutions, and the node's
                // type. All three are applied to the temporary node, so they
                // will unify. Recursive bounds will also be resolved here.
                copy.insert([Constraint::Instantiation(Instantiation {
                    substitutions,
                    constraints: vec![Constraint::Ty(temp_node, Ty::Of(instance))],
                })]);

                if copy.error {
                    // Bound and instance constraints didn't unify; try the next
                    // candidate.
                    continue;
                }

                candidates.push((instance, copy));
            }

            if candidates.len() != 1 {
                self.db.borrow_mut().flag_unresolved(temp_node, bound);
                continue;
            }

            let (instance, copy) = candidates.into_iter().next().unwrap();

            // Incorporate the resolved types from the selected instance
            *self = copy;

            self.progress.set();
            self.db
                .borrow_mut()
                .flag_resolved(temp_node, bound, instance);
        }

        self.progress.take()
    }

    fn fill_with_unknown(&mut self) -> Progress {
        // Give all nodes that aren't in a group at this point a type of
        // `Unknown`.

        let nodes = self
            .db
            .borrow()
            .typed_nodes()
            .filter(|&node| self.try_key_for_node(node).is_none())
            .collect::<Vec<_>>();

        for node in nodes {
            self.unify_tys(&mut Ty::Of(node), &mut Ty::Unknown(node))
                .unwrap();

            self.progress.set();
        }

        self.progress.take()
    }
}

impl<Db: crate::Db> Solver<'_, Db> {
    fn key_for_node(&mut self, node: Db::Node) -> GroupKey<Db> {
        self.keys
            .key_for_node(node, || self.unify.new_key(Group::new(node)))
    }

    fn try_key_for_node(&self, node: Db::Node) -> Option<GroupKey<Db>> {
        self.keys.try_key_for_node(node)
    }

    fn node_for_key(&self, key: GroupKey<Db>) -> Db::Node {
        self.keys.node_for_key(key)
    }
}

impl<Db: crate::Db> Solver<'_, Db> {
    fn try_apply_ty(&self, ty: &mut Ty<Db>, unify: &mut InPlaceUnificationTable<GroupKey<Db>>) {
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

    fn apply_ty(&mut self, ty: &mut Ty<Db>) {
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

    fn apply_instantiation(&mut self, mut instantiation: Instantiation<Db>) -> Vec<Constraint<Db>> {
        let (constraints, requeue): (Vec<_>, Vec<_>) = instantiation
            .constraints
            .into_iter()
            .partition_map(|mut constraint| {
                use itertools::Either::*;

                let mut incomplete = false;
                constraint.traverse_mut(&mut |ty| {
                    self.try_apply_ty(ty, &mut self.unify.clone());

                    // If the type refers to other nodes (definitions) that we
                    // didn't substitute in, it needs to wait for those
                    // definitions to have types, so requeue it.
                    ty.traverse(&mut |ty| {
                        if let Ty::Of(node) = ty {
                            if !instantiation.substitutions.0.contains_key(node) {
                                incomplete = true;
                            }
                        }
                    });
                });

                if incomplete {
                    Right(constraint)
                } else {
                    Left(constraint)
                }
            });

        if !requeue.is_empty() {
            self.queue.push(Constraint::Instantiation(Instantiation {
                substitutions: instantiation.substitutions.clone(),
                constraints: requeue,
            }));
        }

        constraints
            .into_iter()
            .map(|mut constraint| {
                constraint.traverse_mut(&mut |ty| {
                    self.instantiate_applied_ty(ty, &mut instantiation.substitutions);
                });

                constraint
            })
            .collect()
    }

    fn instantiate_applied_ty(&mut self, ty: &mut Ty<Db>, substitutions: &mut Substitutions<Db>) {
        ty.traverse_mut(&mut |ty| {
            if let Ty::Parameter(parameter) = ty {
                if let Some(substitution) = substitutions.0.get(parameter).cloned() {
                    *ty = substitution;
                } else {
                    let copy = self.db.borrow_mut().clone_node(*parameter);
                    substitutions.0.insert(*parameter, Ty::Of(copy));
                    *ty = Ty::Of(copy);
                }
            }
        });
    }

    fn unify_tys(&mut self, left: &mut Ty<Db>, right: &mut Ty<Db>) -> Result<(), ()> {
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
                let existing = self.groups.insert(key, other.clone());
                assert!(existing.is_none());

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

    fn unify_nodes(&mut self, left_node: &mut Db::Node, right_node: &mut Db::Node) {
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
