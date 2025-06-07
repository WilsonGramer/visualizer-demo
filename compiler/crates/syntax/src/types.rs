use crate::tokens::{TypeName, TypeParameterName};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Type {
    #[tree_sitter(rule = "placeholder_type")]
    Placeholder(PlaceholderType),

    #[tree_sitter(rule = "unit_type")]
    Unit(UnitType),

    #[tree_sitter(rule = "named_type")]
    Named(NamedType),

    #[tree_sitter(rule = "block_type")]
    Block(BlockType),

    #[tree_sitter(rule = "function_type")]
    Function(FunctionType),

    #[tree_sitter(rule = "parameter_type")]
    Parameter(ParameterType),

    #[tree_sitter(rule = "tuple_type")]
    Tuple(TupleType),
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct PlaceholderType {
    pub range: Range<usize>,
}

/// ```wipple
/// value
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct ParameterType {
    pub range: Range<usize>,
    pub name: TypeParameterName,
}

/// ```wipple
/// Maybe Number
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct NamedType {
    pub range: Range<usize>,
    pub name: TypeName,
    pub parameters: Vec<Type>,
}

/// ```wipple
/// (Maybe Number) Number -> ()
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct FunctionType {
    pub range: Range<usize>,
    pub inputs: Vec<Type>,
    pub output: Box<Type>,
}

/// ```wipple
/// {Number}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct BlockType {
    pub range: Range<usize>,
    pub output: Box<Type>,
}

/// ```wipple
/// ()
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct UnitType {
    pub range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TupleType {
    pub range: Range<usize>,
    pub elements: Vec<Type>,
}
