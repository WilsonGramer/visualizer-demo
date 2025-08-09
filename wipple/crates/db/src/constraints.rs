use crate::{Db, FactValue, NodeId};
use std::{fmt::Debug, sync::Arc};
use visualizer::Constraint;

pub type LazyConstraint = Arc<dyn Fn(NodeId) -> Constraint<Db> + Send + Sync>;

#[derive(Clone, Default)]
pub struct LazyConstraints(pub Vec<LazyConstraint>);

impl LazyConstraints {
    pub fn resolve_for(&self, node: NodeId) -> Vec<Constraint<Db>> {
        self.0.iter().map(|f| f(node)).collect()
    }
}

impl Debug for LazyConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LazyConstraints").finish_non_exhaustive()
    }
}

impl PartialEq for LazyConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().zip(&other.0).all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

impl Eq for LazyConstraints {}

impl FactValue for LazyConstraints {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(format!("{self:?}"))
    }
}
