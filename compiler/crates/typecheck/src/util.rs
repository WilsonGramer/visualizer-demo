use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::Range,
    rc::Rc,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fact {
    Marker(Rc<str>),
    Str(Rc<str>, Rc<str>),
    Node(NodeId, Rc<str>),
}

impl Fact {
    pub fn marker(name: impl AsRef<str>) -> Self {
        Fact::Marker(Rc::from(name.as_ref()))
    }

    pub fn with_str(name: impl AsRef<str>, str: impl AsRef<str>) -> Self {
        Fact::Str(Rc::from(name.as_ref()), Rc::from(str.as_ref()))
    }

    pub fn with_node(node: NodeId, relation: impl AsRef<str>) -> Self {
        Fact::Node(node, Rc::from(relation.as_ref()))
    }
}

impl Fact {
    pub fn should_ignore(&self) -> bool {
        match self {
            Fact::Marker(name) | Fact::Str(name, _) => name.starts_with('_'),
            _ => false,
        }
    }
}

impl Display for Fact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Fact::Marker(name) => write!(f, "{name}"),
            Fact::Str(name, str) => write!(f, "{name}({str:?})"),
            Fact::Node(node, relation) => write!(f, "{node:?} as {relation:?}"),
        }
    }
}
