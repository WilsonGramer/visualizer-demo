use crate::{
    Type,
    tokens::{Number, Text, VariableName, VariantName},
};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Pattern {
    #[tree_sitter(rule = "unit_pattern")]
    Unit(UnitPattern),

    #[tree_sitter(rule = "wildcard_pattern")]
    Wildcard(WildcardPattern),

    #[tree_sitter(rule = "variable_pattern")]
    Variable(VariablePattern),

    #[tree_sitter(rule = "number_pattern")]
    Number(NumberPattern),

    #[tree_sitter(rule = "text_pattern")]
    Text(TextPattern),

    #[tree_sitter(rule = "destructure_pattern")]
    Destructure(DestructurePattern),

    #[tree_sitter(rule = "set_pattern")]
    Set(SetPattern),

    #[tree_sitter(rule = "variant_pattern")]
    Variant(VariantPattern),

    #[tree_sitter(rule = "or_pattern")]
    Or(OrPattern),

    #[tree_sitter(rule = "tuple_pattern")]
    Tuple(TuplePattern),

    #[tree_sitter(rule = "annotate_pattern")]
    Annotate(AnnotatePattern),
}

impl Pattern {
    pub fn range(&self) -> &Range<usize> {
        match self {
            Pattern::Unit(pattern) => &pattern.range,
            Pattern::Wildcard(pattern) => &pattern.range,
            Pattern::Variable(pattern) => &pattern.range,
            Pattern::Number(pattern) => &pattern.range,
            Pattern::Text(pattern) => &pattern.range,
            Pattern::Destructure(pattern) => &pattern.range,
            Pattern::Set(pattern) => &pattern.range,
            Pattern::Variant(pattern) => &pattern.range,
            Pattern::Or(pattern) => &pattern.range,
            Pattern::Tuple(pattern) => &pattern.range,
            Pattern::Annotate(pattern) => &pattern.range,
        }
    }
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct WildcardPattern {
    pub range: Range<usize>,
}

/// ```wipple
/// x
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct VariablePattern {
    pub range: Range<usize>,
    pub variable: VariableName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct NumberPattern {
    pub range: Range<usize>,
    pub value: Number,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TextPattern {
    pub range: Range<usize>,
    pub value: Text,
}

/// ```wipple
/// {x : y}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct DestructurePattern {
    pub range: Range<usize>,
    pub fields: Vec<DestructurePatternField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct DestructurePatternField {
    pub range: Range<usize>,
    pub name: VariableName,
    pub value: Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct UnitPattern {
    pub range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TuplePattern {
    pub range: Range<usize>,
    pub elements: Vec<Pattern>,
}

/// ```wipple
/// x or y or z
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct OrPattern {
    pub range: Range<usize>,
    pub left: Box<Pattern>,
    pub right: Box<Pattern>,
}

/// ```wipple
/// set x
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct SetPattern {
    pub range: Range<usize>,
    pub variable: VariableName,
}

/// ```wipple
/// Some x y z
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct VariantPattern {
    pub range: Range<usize>,
    pub variant: VariantName,
    pub elements: Vec<Pattern>,
}

/// ```wipple
/// (x :: Number)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct AnnotatePattern {
    pub range: Range<usize>,
    pub left: Box<Pattern>,
    pub right: Type,
}
