use std::{
    any::{Any, TypeId},
    fmt::Debug,
    ops::Range,
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub usize);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub path: Arc<str>,
    pub range: Range<usize>,
    pub start_line_col: (usize, usize),
    pub end_line_col: (usize, usize),
}

impl Span {
    pub fn root(path: Arc<str>) -> Self {
        Span {
            path,
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

pub trait Rule: Copy + Any + 'static {
    fn init() -> Self;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn name(&self) -> &'static str;

    fn erased(self) -> AnyRule {
        AnyRule::new(self)
    }
}

#[macro_export]
macro_rules! rule {
    ($($(#[$meta:meta])+ $name:ident;)*) => {
        pub mod rule {
            $(
                $(#[$meta])+
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                pub struct $name;

                impl $crate::Rule for $name {
                    fn init() -> Self {
                        $name
                    }

                    fn name(&self) -> &'static str {
                        stringify!($name)
                    }
                }
            )*
        }
    };
}

#[derive(Clone, Copy)]
pub struct AnyRule {
    type_id: TypeId,
    name: &'static str,
}

impl AnyRule {
    pub fn new<R: Rule>(rule: R) -> Self {
        AnyRule {
            type_id: Rule::type_id(&rule),
            name: Rule::name(&rule),
        }
    }

    pub fn is<R: Rule>(&self) -> bool {
        let type_id = TypeId::of::<R>();
        type_id == TypeId::of::<AnyRule>() || type_id == self.type_id
    }
}

impl Rule for AnyRule {
    fn init() -> Self {
        panic!("`AnyRule` wraps other rules and cannot be created directly")
    }

    fn type_id(&self) -> TypeId {
        self.type_id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn erased(self) -> AnyRule {
        self
    }
}

impl Debug for AnyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl PartialEq for AnyRule {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for AnyRule {}

impl PartialOrd for AnyRule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AnyRule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl std::hash::Hash for AnyRule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}
