use crate::{
    tokens::{TypeName, TypeParameterName},
    types::{ParameterType, Type},
};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Constraint {
    #[tree_sitter(rule = "bound_constraint")]
    Bound(BoundConstraint),

    #[tree_sitter(rule = "infer_constraint")]
    Infer(InferConstraint),

    #[tree_sitter(rule = "default_constraint")]
    Default(DefaultConstraint),
}

/// ```wipple
/// (Foo value)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct BoundConstraint {
    pub range: Range<usize>,
    pub r#trait: TypeName,
    pub parameter: ParameterType,
}

/// ```wipple
/// (infer value)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct InferConstraint {
    pub range: Range<usize>,
    pub parameter: TypeParameterName,
}

/// ```wipple
/// (value : Number)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct DefaultConstraint {
    pub range: Range<usize>,
    pub parameter: TypeParameterName,
    pub value: Type,
}
