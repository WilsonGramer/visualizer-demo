use crate::{
    Parse, Range, Rule, pest_enum,
    tokens::{TypeName, TypeParameterName},
};
use pest_ast::FromPest;

pest_enum! {
    #[parenthesized = ParenthesizedType]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Type {
        Placeholder(PlaceholderType),
        Unit(UnitType),
        Named(NamedType),
        Parameterized(ParameterizedType),
        Block(BlockType),
        Function(FunctionType),
        Parameter(ParameterType),
        Tuple(TupleType),
    }
}

impl Parse for Type {
    const RULE: crate::Rule = Rule::r#type;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parenthesized_type))]
pub struct ParenthesizedType(pub Type);

impl Type {
    pub fn range(&self) -> Range {
        match self {
            Type::Placeholder(ty) => ty.range,
            Type::Unit(ty) => ty.range,
            Type::Named(ty) => ty.range,
            Type::Parameterized(ty) => ty.range,
            Type::Block(ty) => ty.range,
            Type::Function(ty) => ty.range,
            Type::Parameter(ty) => ty.range,
            Type::Tuple(ty) => ty.range,
        }
    }
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::placeholder_type))]
pub struct PlaceholderType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

/// ```wipple
/// value
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parameter_type))]
pub struct ParameterType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: TypeParameterName,
}

/// ```wipple
/// Number
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::named_type))]
pub struct NamedType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: TypeName,
}

/// ```wipple
/// Maybe Number
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parameterized_type))]
pub struct ParameterizedType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: TypeName,
    pub parameters: Vec<ParameterizedTypeElement>,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parameterized_type_element))]
pub struct ParameterizedTypeElement(pub Type);

/// ```wipple
/// (Maybe Number) Number -> ()
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::function_type))]
pub struct FunctionType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub inputs: FunctionTypeInputs,
    pub output: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::function_type_inputs))]
pub struct FunctionTypeInputs(pub Vec<Type>);

/// ```wipple
/// {Number}
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::block_type))]
pub struct BlockType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub output: Box<Type>,
}

/// ```wipple
/// ()
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::unit_type))]
pub struct UnitType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::tuple_type))]
pub struct TupleType {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub elements: Vec<Type>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_placeholder_type() {
        assert_eq!(
            Type::parse("_").unwrap(),
            Type::Placeholder(PlaceholderType { range: Range::None })
        );
    }

    #[test]
    fn test_unit_type() {
        assert_eq!(
            Type::parse("()").unwrap(),
            Type::Unit(UnitType { range: Range::None })
        );
    }

    #[test]
    fn test_simple_named_type() {
        assert_eq!(
            Type::parse("Number").unwrap(),
            Type::Named(NamedType {
                range: Range::None,
                name: TypeName {
                    range: Range::None,
                    value: String::from("Number")
                },
            })
        );
    }

    #[test]
    fn test_complex_named_type() {
        assert_eq!(
            Type::parse("Maybe Number").unwrap(),
            Type::Parameterized(ParameterizedType {
                range: Range::None,
                name: TypeName {
                    range: Range::None,
                    value: String::from("Maybe")
                },
                parameters: vec![ParameterizedTypeElement(Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("Number")
                    },
                }))],
            })
        );
    }

    #[test]
    fn test_block_type() {
        assert_eq!(
            Type::parse("{Number}").unwrap(),
            Type::Block(BlockType {
                range: Range::None,
                output: Box::new(Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("Number")
                    },
                })),
            })
        );
    }

    #[test]
    fn test_single_input_function_type() {
        assert_eq!(
            Type::parse("Number -> ()").unwrap(),
            Type::Function(FunctionType {
                range: Range::None,
                inputs: FunctionTypeInputs(vec![Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("Number")
                    },
                })]),
                output: Box::new(Type::Unit(UnitType { range: Range::None })),
            })
        );
    }

    #[test]
    fn test_multi_input_function_type() {
        assert_eq!(
            Type::parse("Number Number -> ()").unwrap(),
            Type::Function(FunctionType {
                range: Range::None,
                inputs: FunctionTypeInputs(vec![
                    Type::Named(NamedType {
                        range: Range::None,
                        name: TypeName {
                            range: Range::None,
                            value: String::from("Number")
                        },
                    }),
                    Type::Named(NamedType {
                        range: Range::None,
                        name: TypeName {
                            range: Range::None,
                            value: String::from("Number")
                        },
                    }),
                ]),
                output: Box::new(Type::Unit(UnitType { range: Range::None })),
            })
        );
    }

    #[test]
    fn test_complex_input_function_type() {
        assert_eq!(
            Type::parse("(Maybe Number) Number -> ()").unwrap(),
            Type::Function(FunctionType {
                range: Range::None,
                inputs: FunctionTypeInputs(vec![
                    Type::Parameterized(ParameterizedType {
                        range: Range::None,
                        name: TypeName {
                            range: Range::None,
                            value: String::from("Maybe")
                        },
                        parameters: vec![ParameterizedTypeElement(Type::Named(NamedType {
                            range: Range::None,
                            name: TypeName {
                                range: Range::None,
                                value: String::from("Number")
                            },
                        }))],
                    }),
                    Type::Named(NamedType {
                        range: Range::None,
                        name: TypeName {
                            range: Range::None,
                            value: String::from("Number")
                        },
                    }),
                ]),
                output: Box::new(Type::Unit(UnitType { range: Range::None })),
            })
        );
    }
}
