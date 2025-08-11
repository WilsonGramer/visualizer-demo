use crate::{Db, FactValue};
use std::{
    fmt::{Debug, Display},
    ops::Range,
    sync::Arc,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub path: Arc<str>,
    pub range: Range<usize>,
    pub start_line_col: (usize, usize),
    pub end_line_col: (usize, usize),
}

impl Span {
    pub fn root(path: impl AsRef<str>) -> Self {
        Span {
            path: Arc::from(path.as_ref()),
            range: 0..0,
            start_line_col: (0, 0),
            end_line_col: (0, 0),
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}.{}-{}.{}",
            self.path,
            self.start_line_col.0,
            self.start_line_col.1,
            self.end_line_col.0,
            self.end_line_col.1
        )
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl FactValue for Span {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(self.to_string())
    }
}
