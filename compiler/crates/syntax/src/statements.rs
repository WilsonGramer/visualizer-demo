use crate::{
    Attribute, TypeParameterName,
    constraints::Constraint,
    expressions::Expression,
    patterns::Pattern,
    tokens::{Comment, TypeName, VariableName, VariantName},
    types::Type,
};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Statement {
    #[tree_sitter(rule = "constant_definition_statement")]
    ConstantDefinition(ConstantDefinitionStatement),

    #[tree_sitter(rule = "expression_statement")]
    Expression(ExpressionStatement),

    #[tree_sitter(rule = "assignment_statement")]
    Assignment(AssignmentStatement),

    #[tree_sitter(rule = "type_definition_statement")]
    TypeDefinition(TypeDefinitionStatement),

    #[tree_sitter(rule = "trait_definition_statement")]
    TraitDefinition(TraitDefinitionStatement),

    #[tree_sitter(rule = "instance_definition_statement")]
    InstanceDefinition(InstanceDefinitionStatement),
}

/// ```wipple
/// Foo : value => type
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TypeDefinitionStatement {
    pub range: Range<usize>,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: TypeName,
    pub parameters: Vec<TypeParameterName>,
    pub representation: Option<TypeRepresentation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum TypeRepresentation {
    #[tree_sitter(rule = "structure_representation")]
    Structure(StructureTypeRepresentation),

    #[tree_sitter(rule = "enumeration_representation")]
    Enumeration(EnumerationTypeRepresentation),
}

/// ```wipple
/// {
///     a :: A
///     b :: B
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct StructureTypeRepresentation {
    pub range: Range<usize>,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct FieldDefinition {
    pub range: Range<usize>,
    pub name: VariableName,
    pub r#type: Type,
}

/// ```wipple
/// {
///     Some Number
///     None
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct EnumerationTypeRepresentation {
    pub range: Range<usize>,
    pub variants: Vec<VariantDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct VariantDefinition {
    pub range: Range<usize>,
    pub name: VariantName,
    pub elements: Vec<Type>,
}

/// ```wipple
/// Foo : value => trait (value -> Number)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TraitDefinitionStatement {
    pub range: Range<usize>,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: TypeName,
    pub parameters: Vec<TypeParameterName>,
    pub r#type: Option<Type>,
}

/// ```wipple
/// show :: value -> Unit where (Show value)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct ConstantDefinitionStatement {
    pub range: Range<usize>,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: VariableName,
    pub r#type: Type,
    pub constraints: Vec<Constraint>,
}

/// ```wipple
/// instance (Foo (Maybe value)) where (Foo value) : 3.14
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct InstanceDefinitionStatement {
    pub range: Range<usize>,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub r#trait: TypeName,
    pub parameter: Type,
    pub constraints: Vec<Constraint>,
    pub value: Option<Expression>,
}

/// ```wipple
/// Some x y z : ()
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct AssignmentStatement {
    pub range: Range<usize>,
    pub pattern: Pattern,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct ExpressionStatement {
    pub range: Range<usize>,
    pub expression: Expression,
}
