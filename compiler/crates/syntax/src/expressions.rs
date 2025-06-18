use crate::{
    BinaryOperator, TypeName,
    patterns::Pattern,
    statements::Statement,
    tokens::{Number, Text, VariableName},
    types::Type,
};
use derive_tree_sitter::FromNode;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub enum Expression {
    #[tree_sitter(rule = "placeholder_expression")]
    Placeholder(PlaceholderExpression),

    #[tree_sitter(rule = "variable_name_expression")]
    VariableName(VariableNameExpression),

    #[tree_sitter(rule = "type_name_expression")]
    TypeName(TypeNameExpression),

    #[tree_sitter(rule = "number_expression")]
    Number(NumberExpression),

    #[tree_sitter(rule = "text_expression")]
    Text(TextExpression),

    #[tree_sitter(rule = "formatted_text_expression")]
    FormattedText(FormattedTextExpression),

    #[tree_sitter(rule = "structure_expression")]
    Structure(StructureExpression),

    #[tree_sitter(rule = "block_expression")]
    Block(BlockExpression),

    #[tree_sitter(rule = "unit_expression")]
    Unit(UnitExpression),

    #[tree_sitter(rule = "call_expression")]
    Call(CallExpression),

    #[tree_sitter(rule = "do_expression")]
    Do(DoExpression),

    #[tree_sitter(rule = "when_expression")]
    When(WhenExpression),

    #[tree_sitter(rule = "intrinsic_expression")]
    Intrinsic(IntrinsicExpression),

    #[tree_sitter(rule = "annotate_expression")]
    Annotate(AnnotateExpression),

    #[tree_sitter(rule = "as_expression")]
    As(AsExpression),

    #[tree_sitter(rule = "to_expression")]
    To(BinaryExpression),

    #[tree_sitter(rule = "by_expression")]
    By(BinaryExpression),

    #[tree_sitter(rule = "power_expression")]
    Power(BinaryExpression),

    #[tree_sitter(rule = "multiply_expression")]
    Multiply(BinaryExpression),

    #[tree_sitter(rule = "divide_expression")]
    Divide(BinaryExpression),

    #[tree_sitter(rule = "remainder_expression")]
    Remainder(BinaryExpression),

    #[tree_sitter(rule = "add_expression")]
    Add(BinaryExpression),

    #[tree_sitter(rule = "subtract_expression")]
    Subtract(BinaryExpression),

    #[tree_sitter(rule = "less_than_expression")]
    LessThan(BinaryExpression),

    #[tree_sitter(rule = "less_than_or_equal_expression")]
    LessThanOrEqual(BinaryExpression),

    #[tree_sitter(rule = "greater_than_expression")]
    GreaterThan(BinaryExpression),

    #[tree_sitter(rule = "greater_than_or_equal_expression")]
    GreaterThanOrEqual(BinaryExpression),

    #[tree_sitter(rule = "equal_expression")]
    Equal(BinaryExpression),

    #[tree_sitter(rule = "not_equal_expression")]
    NotEqual(BinaryExpression),

    #[tree_sitter(rule = "is_expression")]
    Is(IsExpression),

    #[tree_sitter(rule = "and_expression")]
    And(BinaryExpression),

    #[tree_sitter(rule = "or_expression")]
    Or(BinaryExpression),

    #[tree_sitter(rule = "apply_expression")]
    Apply(ApplyExpression),

    #[tree_sitter(rule = "tuple_expression")]
    Tuple(TupleExpression),

    #[tree_sitter(rule = "collection_expression")]
    Collection(CollectionExpression),

    #[tree_sitter(rule = "function_expression")]
    Function(FunctionExpression),
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct PlaceholderExpression {
    pub range: Range<usize>,
}

/// ```wipple
/// foo
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct VariableNameExpression {
    pub range: Range<usize>,
    pub variable: VariableName,
}

/// ```wipple
/// foo
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TypeNameExpression {
    pub range: Range<usize>,
    pub r#type: TypeName,
}

/// ```wipple
/// 3.14
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct NumberExpression {
    pub range: Range<usize>,
    pub value: Number,
}

/// ```wipple
/// "abc"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TextExpression {
    pub range: Range<usize>,
    pub value: Text,
}

/// ```wipple
/// {
///     a : b
///     c : d
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct StructureExpression {
    pub range: Range<usize>,
    pub fields: Vec<StructureExpressionField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct StructureExpressionField {
    pub range: Range<usize>,
    pub name: VariableName,
    pub value: Expression,
}

/// ```wipple
/// {foo}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct BlockExpression {
    pub range: Range<usize>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct UnitExpression {
    pub range: Range<usize>,
}

/// ```wipple
/// "Hello, _!" name
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct FormattedTextExpression {
    pub range: Range<usize>,
    pub text: Text,
    pub input: Box<Expression>,
}

/// ```wipple
/// f x y
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct CallExpression {
    pub range: Range<usize>,
    pub function: Box<Expression>,
    pub inputs: Vec<Expression>,
}

/// ```wipple
/// do foo
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct DoExpression {
    pub range: Range<usize>,
    pub input: Box<Expression>,
}

/// ```wipple
/// when x {
///     a -> b
///     c -> d
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct WhenExpression {
    pub range: Range<usize>,
    pub input: Box<Expression>,
    pub arms: Vec<Arm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct Arm {
    pub range: Range<usize>,
    pub pattern: Pattern,
    pub value: Expression,
}

/// ```wipple
/// intrinsic "message" x y
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct IntrinsicExpression {
    pub range: Range<usize>,
    pub name: Text,
    pub inputs: Vec<Expression>,
}

/// ```wipple
/// (3.14 :: Number)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct AnnotateExpression {
    pub range: Range<usize>,
    pub left: Box<Expression>,
    pub right: Type,
}

/// ```wipple
/// a b . c d
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct ApplyExpression {
    pub range: Range<usize>,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

/// ```wipple
/// x as T
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct AsExpression {
    pub range: Range<usize>,
    pub left: Box<Expression>,
    pub right: Type,
}

/// ```wipple
/// x + y
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct BinaryExpression {
    pub range: Range<usize>,
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

/// ```wipple
/// x is Some 3.14
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct IsExpression {
    pub range: Range<usize>,
    pub left: Box<Expression>,
    pub right: Pattern,
}

/// ```wipple
/// (
///     a ;
///     b ;
///     c ;
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct TupleExpression {
    pub range: Range<usize>,
    pub elements: Vec<Expression>,
}

/// ```wipple
/// (
///     a ,
///     b ,
///     c ,
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct CollectionExpression {
    pub range: Range<usize>,
    pub elements: Vec<Expression>,
}

/// ```wipple
/// (X y) -> z
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct FunctionExpression {
    pub range: Range<usize>,
    pub inputs: Vec<Pattern>,
    pub output: Box<Expression>,
}
