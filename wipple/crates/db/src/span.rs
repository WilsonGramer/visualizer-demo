use crate::{Db, FactValue};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Range};

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

impl FactValue for Span {
    fn eq(&self, other: &dyn FactValue) -> bool {
        other
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn display(&self, _db: &Db) -> Option<String> {
        Some(format!("{self:?}"))
    }
}
