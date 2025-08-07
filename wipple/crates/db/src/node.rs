use crate::{Db, FactValue};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl FactValue for NodeId {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(format!("{self:?}"))
    }
}
