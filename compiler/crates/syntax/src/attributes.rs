use crate::tokens::{AttributeName, Text};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Attribute {
    #[tree_sitter(rule = "name_attribute")]
    Name(NameAttribute),

    #[tree_sitter(rule = "assign_attribute")]
    Assign(AssignAttribute),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct NameAttribute {
    pub range: Range<usize>,
    pub name: AttributeName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct AssignAttribute {
    pub range: Range<usize>,
    pub name: AttributeName,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum AttributeValue {
    #[tree_sitter(rule = "text")]
    Text(Text),
}
