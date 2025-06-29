use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Range};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub namespace: Option<u32>,
    pub index: u32,
}

impl NodeId {
    pub fn with_namespace(mut self, namespace: u32) -> Self {
        let existing = self.namespace.replace(namespace);
        assert!(existing.is_none());

        self
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.namespace {
            Some(namespace) => write!(f, "NodeId({} in namespace {})", self.index, namespace),
            None => write!(f, "NodeId({})", self.index),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub path: String,
    pub range: Range<usize>,
    pub start_line_col: (usize, usize),
    pub end_line_col: (usize, usize),
}

impl Span {
    pub fn root(path: impl AsRef<str>) -> Self {
        Span {
            path: path.as_ref().to_string(),
            range: 0..0,
            start_line_col: (0, 0),
            end_line_col: (0, 0),
        }
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.path, self.start_line_col.0, self.start_line_col.1
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rule {
    pub name: &'static str,
}

impl Rule {
    pub const fn new(name: &'static str) -> Self {
        Rule { name }
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
