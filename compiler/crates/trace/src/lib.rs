use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub usize);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rule {
    pub name: &'static str,
    pub hidden: bool,
}

impl Rule {
    pub const fn new(name: &'static str) -> Self {
        Rule {
            name,
            hidden: false,
        }
    }

    pub const fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}
