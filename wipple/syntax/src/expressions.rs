use crate::{
    Parse, Range, Rule, Statements, TypeName,
    patterns::Pattern,
    pest_enum,
    tokens::{Number, Text, VariableName},
    types::Type,
};
use from_pest::FromPest;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_ast::FromPest;

pest_enum! {
    #[parenthesized = ParenthesizedExpression]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Expression {
        Function(FunctionExpression),
        Tuple(TupleExpression),
        Collection(CollectionExpression),
        Is(IsExpression),
        As(AsExpression),
        Annotate(AnnotateExpression),
        Binary(BinaryExpression),
        FormattedText(FormattedTextExpression),
        Call(CallExpression),
        Do(DoExpression),
        When(WhenExpression),
        Intrinsic(IntrinsicExpression),
        Placeholder(PlaceholderExpression),
        Variable(VariableExpression),
        Trait(TraitExpression),
        Number(NumberExpression),
        Text(TextExpression),
        Structure(StructureExpression),
        Block(BlockExpression),
        Unit(UnitExpression),
    }
}

impl Parse for Expression {
    const RULE: crate::Rule = Rule::expression;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parenthesized_expression))]
pub struct ParenthesizedExpression(pub Expression);

impl Expression {
    pub fn range(&self) -> Range {
        match self {
            Expression::Function(expression) => expression.range,
            Expression::Tuple(expression) => expression.range,
            Expression::Collection(expression) => expression.range,
            Expression::Is(expression) => expression.range,
            Expression::As(expression) => expression.range,
            Expression::Annotate(expression) => expression.range,
            Expression::Binary(expression) => expression.range(),
            Expression::FormattedText(expression) => expression.range,
            Expression::Call(expression) => expression.range,
            Expression::Do(expression) => expression.range,
            Expression::When(expression) => expression.range,
            Expression::Intrinsic(expression) => expression.range,
            Expression::Placeholder(expression) => expression.range,
            Expression::Variable(expression) => expression.range,
            Expression::Trait(expression) => expression.range,
            Expression::Number(expression) => expression.range,
            Expression::Text(expression) => expression.range,
            Expression::Structure(expression) => expression.range,
            Expression::Block(expression) => expression.range,
            Expression::Unit(expression) => expression.range,
        }
    }
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::placeholder_expression))]
pub struct PlaceholderExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

/// ```wipple
/// foo
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variable_expression))]
pub struct VariableExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub variable: VariableName,
}

/// ```wipple
/// Foo
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::trait_expression))]
pub struct TraitExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub r#type: TypeName,
}

/// ```wipple
/// 3.14
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::number_expression))]
pub struct NumberExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub value: Number,
}

/// ```wipple
/// "abc"
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::text_expression))]
pub struct TextExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub value: Text,
}

/// ```wipple
/// {
///     a : b
///     c : d
/// }
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::structure_expression))]
pub struct StructureExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub fields: StructureExpressionFields,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::structure_expression_fields))]
pub struct StructureExpressionFields(pub Vec<StructureExpressionField>);

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::structure_expression_field))]
pub struct StructureExpressionField {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: VariableName,
    pub value: Expression,
}

/// ```wipple
/// {foo}
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::block_expression))]
pub struct BlockExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub statements: Statements,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::unit_expression))]
pub struct UnitExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

/// ```wipple
/// "Hello, _!" name
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::formatted_text_expression))]
pub struct FormattedTextExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub text: Text,
    pub input: Box<Expression>,
}

/// ```wipple
/// f x y
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::call_expression))]
pub struct CallExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub function: Box<Expression>,
    pub inputs: Vec<Expression>,
}

/// ```wipple
/// do foo
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::do_expression))]
pub struct DoExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub input: Box<Expression>,
}

/// ```wipple
/// when x {
///     a -> b
///     c -> d
/// }
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::when_expression))]
pub struct WhenExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub input: Box<Expression>,
    pub arms: Arms,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::arms))]
pub struct Arms(pub Vec<Arm>);

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::arm))]
pub struct Arm {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub pattern: Pattern,
    pub value: Expression,
}

/// ```wipple
/// intrinsic "message" x y
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::intrinsic_expression))]
pub struct IntrinsicExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: Text,
    pub inputs: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryExpression {
    To(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    By(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Power(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Multiply(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Divide(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Remainder(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Add(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Subtract(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    LessThan(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    LessThanOrEqual(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    GreaterThan(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    GreaterThanOrEqual(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Equal(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    NotEqual(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    And(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Or(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
    Apply(BinaryExpressionInner<Box<Expression>, Box<Expression>>),
}

impl BinaryExpression {
    pub fn range(&self) -> Range {
        match self {
            BinaryExpression::To(expression) => expression.range,
            BinaryExpression::By(expression) => expression.range,
            BinaryExpression::Power(expression) => expression.range,
            BinaryExpression::Multiply(expression) => expression.range,
            BinaryExpression::Divide(expression) => expression.range,
            BinaryExpression::Remainder(expression) => expression.range,
            BinaryExpression::Add(expression) => expression.range,
            BinaryExpression::Subtract(expression) => expression.range,
            BinaryExpression::LessThan(expression) => expression.range,
            BinaryExpression::LessThanOrEqual(expression) => expression.range,
            BinaryExpression::GreaterThan(expression) => expression.range,
            BinaryExpression::GreaterThanOrEqual(expression) => expression.range,
            BinaryExpression::Equal(expression) => expression.range,
            BinaryExpression::NotEqual(expression) => expression.range,
            BinaryExpression::And(expression) => expression.range,
            BinaryExpression::Or(expression) => expression.range,
            BinaryExpression::Apply(expression) => expression.range,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpressionInner<Left, Right> {
    pub range: Range,
    pub left: Left,
    pub right: Right,
}

impl<'pest> FromPest<'pest> for BinaryExpression {
    type Rule = Rule;
    type FatalError = from_pest::Void;

    fn from_pest(
        pest: &mut pest::iterators::Pairs<'pest, Self::Rule>,
    ) -> Result<Self, from_pest::ConversionError<Self::FatalError>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(from_pest::ConversionError::NoMatch)?;
        if pair.as_rule() != Rule::binary_expression {
            return Err(from_pest::ConversionError::NoMatch);
        }

        let mut pairs = pair.into_inner();

        let ops = [
            (Rule::apply_operator, Assoc::Left),
            (Rule::or_operator, Assoc::Left),
            (Rule::and_operator, Assoc::Left),
            (Rule::equal_operator, Assoc::Left),
            (Rule::compare_operator, Assoc::Left),
            (Rule::add_operator, Assoc::Left),
            (Rule::multiply_operator, Assoc::Left),
            (Rule::power_operator, Assoc::Right),
            (Rule::by_operator, Assoc::Left),
            (Rule::to_operator, Assoc::Left),
        ];

        let pratt = ops
            .into_iter()
            .rev()
            .fold(PrattParser::new(), |pratt, (rule, assoc)| {
                pratt.op(Op::infix(rule, assoc))
            });

        let expression = pratt
            .map_primary(|primary| {
                Expression::from_pest(&mut pest::iterators::Pairs::single(primary))
            })
            .map_infix(|left, operator, right| {
                let (left, right) = (left?, right?);

                let range = Range::from(operator.as_span());
                let operator = operator.as_str();

                let operator = match operator {
                    "." => BinaryExpression::Apply,
                    "or" => BinaryExpression::Or,
                    "and" => BinaryExpression::And,
                    "=" => BinaryExpression::Equal,
                    "/=" => BinaryExpression::NotEqual,
                    "<" => BinaryExpression::LessThan,
                    "<=" => BinaryExpression::LessThanOrEqual,
                    ">" => BinaryExpression::GreaterThan,
                    ">=" => BinaryExpression::GreaterThanOrEqual,
                    "+" => BinaryExpression::Add,
                    "-" => BinaryExpression::Subtract,
                    "*" => BinaryExpression::Multiply,
                    "/" => BinaryExpression::Divide,
                    "%" => BinaryExpression::Remainder,
                    "^" => BinaryExpression::Power,
                    "by" => BinaryExpression::By,
                    "to" => BinaryExpression::To,
                    _ => unreachable!(),
                };

                Ok(Expression::Binary(operator(BinaryExpressionInner {
                    range,
                    left: Box::new(left),
                    right: Box::new(right),
                })))
            })
            .parse(&mut pairs)?;

        let Expression::Binary(expression) = expression else {
            unreachable!();
        };

        *pest = clone;

        Ok(expression)
    }
}

/// ```wipple
/// (
///     a ;
///     b ;
///     c ;
/// )
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::tuple_expression))]
pub struct TupleExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub elements: Vec<Expression>,
}

/// ```wipple
/// (
///     a ,
///     b ,
///     c ,
/// )
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::collection_expression))]
pub struct CollectionExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::is_expression))]
pub struct IsExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub left: Box<Expression>,
    pub right: Pattern,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::as_expression))]
pub struct AsExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub left: Box<Expression>,
    pub right: Type,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::annotate_expression))]
pub struct AnnotateExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub left: Box<Expression>,
    pub right: Type,
}

/// ```wipple
/// (X y) -> z
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::function_expression))]
pub struct FunctionExpression {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub inputs: FunctionExpressionInputs,
    pub output: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::function_expression_inputs))]
pub struct FunctionExpressionInputs(pub Vec<Pattern>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_variable_expression() {
        assert_eq!(
            Expression::parse("foo").unwrap(),
            Expression::Variable(VariableExpression {
                range: Range::None,
                variable: VariableName {
                    range: Range::None,
                    value: String::from("foo")
                },
            })
        );
    }

    #[test]
    fn test_number_expression() {
        assert_eq!(
            Expression::parse("3.14").unwrap(),
            Expression::Number(NumberExpression {
                range: Range::None,
                value: Number {
                    range: Range::None,
                    value: String::from("3.14")
                },
            })
        );
    }

    #[test]
    fn test_text_expression() {
        assert_eq!(
            Expression::parse(r#""abc""#).unwrap(),
            Expression::Text(TextExpression {
                range: Range::None,
                value: Text {
                    range: Range::None,
                    value: String::from("abc")
                },
            })
        );
    }

    #[test]
    fn test_formatted_text_expression() {
        assert_eq!(
            Expression::parse(r#""Hello, _!" name"#).unwrap(),
            Expression::FormattedText(FormattedTextExpression {
                range: Range::None,
                text: Text {
                    range: Range::None,
                    value: String::from("Hello, _!")
                },
                input: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("name")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_structure_expression() {
        assert_eq!(
            Expression::parse(
                r#"{
  a : b
  c : d
}"#
            )
            .unwrap(),
            Expression::Structure(StructureExpression {
                range: Range::None,
                fields: StructureExpressionFields(vec![
                    StructureExpressionField {
                        range: Range::None,
                        name: VariableName {
                            range: Range::None,
                            value: String::from("a")
                        },
                        value: Expression::Variable(VariableExpression {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("b")
                            },
                        }),
                    },
                    StructureExpressionField {
                        range: Range::None,
                        name: VariableName {
                            range: Range::None,
                            value: String::from("c")
                        },
                        value: Expression::Variable(VariableExpression {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("d")
                            },
                        }),
                    },
                ]),
            })
        );
    }

    #[test]
    fn test_block_expression() {
        assert_eq!(
            Expression::parse("{foo}").unwrap(),
            Expression::Block(BlockExpression {
                range: Range::None,
                statements: Statements(vec![Statement::Expression(ExpressionStatement {
                    range: Range::None,
                    comments: Comments(Vec::new()),
                    expression: Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("foo")
                        },
                    }),
                })]),
            })
        );
    }

    #[test]
    fn test_do_expression() {
        assert_eq!(
            Expression::parse("do foo").unwrap(),
            Expression::Do(DoExpression {
                range: Range::None,
                input: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("foo")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_simple_intrinsic_expression() {
        assert_eq!(
            Expression::parse(r#"intrinsic "message""#).unwrap(),
            Expression::Intrinsic(IntrinsicExpression {
                range: Range::None,
                name: Text {
                    range: Range::None,
                    value: String::from("message"),
                },
                inputs: Vec::new(),
            })
        );
    }

    #[test]
    fn test_complex_intrinsic_expression() {
        assert_eq!(
            Expression::parse(r#"intrinsic "message" x y"#).unwrap(),
            Expression::Intrinsic(IntrinsicExpression {
                range: Range::None,
                name: Text {
                    range: Range::None,
                    value: String::from("message"),
                },
                inputs: vec![
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        },
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_when_expression() {
        assert_eq!(
            Expression::parse(
                r#"when x {
  a -> b
  c -> d
}"#
            )
            .unwrap(),
            Expression::When(WhenExpression {
                range: Range::None,
                input: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })),
                arms: Arms(vec![
                    Arm {
                        range: Range::None,
                        pattern: Pattern::Variable(VariablePattern {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("a")
                            }
                        }),
                        value: Expression::Variable(VariableExpression {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("b")
                            },
                        }),
                    },
                    Arm {
                        range: Range::None,
                        pattern: Pattern::Variable(VariablePattern {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("c")
                            }
                        }),
                        value: Expression::Variable(VariableExpression {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::None,
                                value: String::from("d")
                            },
                        }),
                    },
                ]),
            })
        );
    }

    #[test]
    fn test_call_expression() {
        assert_eq!(
            Expression::parse("f x y").unwrap(),
            Expression::Call(CallExpression {
                range: Range::None,
                function: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("f")
                    },
                })),
                inputs: vec![
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        },
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_annotate_expression() {
        assert_eq!(
            Expression::parse("(3.14 :: Number)").unwrap(),
            Expression::Annotate(AnnotateExpression {
                range: Range::None,
                left: Box::new(Expression::Number(NumberExpression {
                    range: Range::None,
                    value: Number {
                        range: Range::None,
                        value: String::from("3.14")
                    },
                })),
                right: Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("Number")
                    },
                }),
            })
        );
    }

    #[test]
    fn test_simple_apply_expression() {
        assert_eq!(
            Expression::parse("x . f").unwrap(),
            Expression::Binary(BinaryExpression::Apply(BinaryExpressionInner {
                range: Range::None,
                left: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })),
                right: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("f")
                    },
                })),
            }))
        );
    }

    #[test]
    fn test_complex_apply_expression() {
        assert_eq!(
            Expression::parse("a b . c d").unwrap(),
            Expression::Binary(BinaryExpression::Apply(BinaryExpressionInner {
                range: Range::None,
                left: Box::new(Expression::Call(CallExpression {
                    range: Range::None,
                    function: Box::new(Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("a")
                        },
                    })),
                    inputs: vec![Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("b")
                        },
                    })],
                })),
                right: Box::new(Expression::Call(CallExpression {
                    range: Range::None,
                    function: Box::new(Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("c")
                        },
                    })),
                    inputs: vec![Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("d")
                        },
                    })],
                })),
            }))
        );
    }

    #[test]
    fn test_as_expression() {
        assert_eq!(
            Expression::parse("x as T").unwrap(),
            Expression::As(AsExpression {
                range: Range::None,
                left: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })),
                right: Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("T")
                    },
                }),
            })
        );
    }

    #[test]
    fn test_add_expression() {
        assert_eq!(
            Expression::parse("a + b").unwrap(),
            Expression::Binary(BinaryExpression::Add(BinaryExpressionInner {
                range: Range::None,
                left: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("a")
                    },
                })),
                right: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("b")
                    },
                })),
            }))
        );
    }

    #[test]
    fn test_empty_collection_expression() {
        assert_eq!(
            Expression::parse("(,)").unwrap(),
            Expression::Collection(CollectionExpression {
                range: Range::None,
                elements: Vec::new()
            })
        );
    }

    #[test]
    fn test_single_line_collection_expression() {
        assert_eq!(
            Expression::parse("a , b , c").unwrap(),
            Expression::Collection(CollectionExpression {
                range: Range::None,
                elements: vec![
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("a")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("b")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("c")
                        },
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_multiline_collection_expression() {
        assert_eq!(
            Expression::parse(
                r#"(
  a ,
  b ,
  c ,
)"#
            )
            .unwrap(),
            Expression::Collection(CollectionExpression {
                range: Range::None,
                elements: vec![
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("a")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("b")
                        },
                    }),
                    Expression::Variable(VariableExpression {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("c")
                        },
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_single_input_function_expression() {
        assert_eq!(
            Expression::parse("x -> y").unwrap(),
            Expression::Function(FunctionExpression {
                range: Range::None,
                inputs: FunctionExpressionInputs(vec![Pattern::Variable(VariablePattern {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })]),
                output: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("y")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_multi_input_function_expression() {
        assert_eq!(
            Expression::parse("x y -> z").unwrap(),
            Expression::Function(FunctionExpression {
                range: Range::None,
                inputs: FunctionExpressionInputs(vec![
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        },
                    }),
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        },
                    })
                ]),
                output: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("z")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_complex_input_function_expression() {
        assert_eq!(
            Expression::parse("(X y) -> z").unwrap(),
            Expression::Function(FunctionExpression {
                range: Range::None,
                inputs: FunctionExpressionInputs(vec![Pattern::Variant(VariantPattern {
                    range: Range::None,
                    variant: VariantName {
                        range: Range::None,
                        value: String::from("X")
                    },
                    elements: vec![VariantPatternElement(Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        },
                    }))],
                })]),
                output: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("z")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_simple_is_expression() {
        assert_eq!(
            Expression::parse("x is None").unwrap(),
            Expression::Is(IsExpression {
                range: Range::None,
                left: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })),
                right: Pattern::Variant(VariantPattern {
                    range: Range::None,
                    variant: VariantName {
                        range: Range::None,
                        value: String::from("None")
                    },
                    elements: Vec::new(),
                }),
            })
        );
    }

    #[test]
    fn test_complex_is_expression() {
        assert_eq!(
            Expression::parse("x is Some 3.14").unwrap(),
            Expression::Is(IsExpression {
                range: Range::None,
                left: Box::new(Expression::Variable(VariableExpression {
                    range: Range::None,
                    variable: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                })),
                right: Pattern::Variant(VariantPattern {
                    range: Range::None,
                    variant: VariantName {
                        range: Range::None,
                        value: String::from("Some")
                    },
                    elements: vec![VariantPatternElement(Pattern::Number(NumberPattern {
                        range: Range::None,
                        value: Number {
                            range: Range::None,
                            value: String::from("3.14")
                        },
                    }))],
                }),
            })
        );
    }
}
