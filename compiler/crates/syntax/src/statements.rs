use crate::{
    Attribute, BoundConstraint, NeverParenthesized, Parse, Range, Rule, TypeParameterName,
    constraints::Constraint,
    expressions::Expression,
    patterns::Pattern,
    pest_enum,
    tokens::{Comment, TypeName, VariableName, VariantName},
    types::Type,
};
use pest_ast::FromPest;

pest_enum! {
    #[parenthesized = NeverParenthesized<Self>]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Statement {
        ConstantDefinition(ConstantDefinitionStatement),
        Expression(ExpressionStatement),
        Assignment(AssignmentStatement),
        TypeDefinition(TypeDefinitionStatement),
        TraitDefinition(TraitDefinitionStatement),
        InstanceDefinition(InstanceDefinitionStatement),
    }
}

impl Parse for Statement {
    const RULE: crate::Rule = Rule::statement;
}

/// ```wipple
/// Foo : value => type
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::type_definition_statement))]
pub struct TypeDefinitionStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: TypeName,
    pub parameters: Option<TypeParameters>,
    pub representation: TypeRepresentation,
}

impl Parse for TypeDefinitionStatement {
    const RULE: crate::Rule = Rule::type_definition_statement;
}

pest_enum! {
    #[parenthesized = NeverParenthesized<Self>]
    #[derive(Debug, Clone, PartialEq)]
    pub enum TypeRepresentation {
        Structure(StructureTypeRepresentation),
        Enumeration(EnumerationTypeRepresentation),
        Marker(MarkerTypeRepresentation),
    }
}

impl Parse for TypeRepresentation {
    const RULE: crate::Rule = Rule::type_representation;
}

/// ```wipple
/// {
///     a :: A
///     b :: B
/// }
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::structure_type_representation))]
pub struct StructureTypeRepresentation {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub fields: Vec<FieldDefinition>,
}

impl Parse for StructureTypeRepresentation {
    const RULE: crate::Rule = Rule::structure_type_representation;
}

/// ```wipple
/// a :: A
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::field_definition))]
pub struct FieldDefinition {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: VariableName,
    pub r#type: Type,
}

impl Parse for FieldDefinition {
    const RULE: crate::Rule = Rule::field_definition;
}

/// ```wipple
/// {
///     Some Number
///     None
/// }
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::enumeration_type_representation))]
pub struct EnumerationTypeRepresentation {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub variants: Vec<VariantDefinition>,
}

impl Parse for EnumerationTypeRepresentation {
    const RULE: crate::Rule = Rule::enumeration_type_representation;
}

/// ```wipple
/// Some Number
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variant_definition))]
pub struct VariantDefinition {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: VariantName,
    pub elements: Vec<VariantDefinitionElement>,
}

impl Parse for VariantDefinition {
    const RULE: crate::Rule = Rule::variant_definition;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variant_definition_element))]
pub struct VariantDefinitionElement(pub Type);

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::marker_type_representation))]
pub struct MarkerTypeRepresentation {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

impl Parse for MarkerTypeRepresentation {
    const RULE: crate::Rule = Rule::marker_type_representation;
}

/// ```wipple
/// Foo : value => trait (value -> Number)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::trait_definition_statement))]
pub struct TraitDefinitionStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: TypeName,
    pub parameters: Option<TypeParameters>,
    pub constraints: TraitConstraints,
}

impl Parse for TraitDefinitionStatement {
    const RULE: crate::Rule = Rule::trait_definition_statement;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::trait_constraints))]
pub struct TraitConstraints {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub r#type: Type,
    pub constraints: Option<Constraints>,
}

impl Parse for TraitConstraints {
    const RULE: crate::Rule = Rule::trait_constraints;
}

/// ```wipple
/// show :: value -> Unit where (Show value)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::constant_definition_statement))]
pub struct ConstantDefinitionStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub name: VariableName,
    pub constraints: ConstantConstraints,
}

impl Parse for ConstantDefinitionStatement {
    const RULE: crate::Rule = Rule::constant_definition_statement;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::constant_constraints))]
pub struct ConstantConstraints {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub r#type: Type,
    pub constraints: Option<Constraints>,
}

impl Parse for ConstantConstraints {
    const RULE: crate::Rule = Rule::constant_constraints;
}

/// ```wipple
/// instance (Foo (Maybe value)) where (Foo value) : 3.14
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::instance_definition_statement))]
pub struct InstanceDefinitionStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub comments: Vec<Comment>,
    pub attributes: Vec<Attribute>,
    pub constraints: InstanceConstraints,
    pub value: Expression,
}

impl Parse for InstanceDefinitionStatement {
    const RULE: crate::Rule = Rule::instance_definition_statement;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::instance_constraints))]
pub struct InstanceConstraints {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub bound: BoundConstraint,
    pub constraints: Option<Constraints>,
}

/// ```wipple
/// Some x y z : ()
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::assignment_statement))]
pub struct AssignmentStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub pattern: Pattern,
    pub value: Expression,
}

impl Parse for AssignmentStatement {
    const RULE: crate::Rule = Rule::assignment_statement;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::expression_statement))]
pub struct ExpressionStatement {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub expression: Expression,
}

impl Parse for ExpressionStatement {
    const RULE: crate::Rule = Rule::expression_statement;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::type_parameters))]
pub struct TypeParameters(pub Vec<TypeParameterName>);

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::constraints))]
pub struct Constraints(pub Vec<Constraint>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_type_definition() {
        assert_eq!(
            Statement::parse("-- Documentation comment\n[foo]\nFoo : type").unwrap(),
            Statement::TypeDefinition(TypeDefinitionStatement {
                range: Range::None,
                comments: vec![Comment {
                    range: Range::None,
                    value: String::from(" Documentation comment")
                }],
                attributes: vec![Attribute {
                    range: Range::None,
                    name: AttributeName {
                        range: Range::None,
                        value: String::from("foo")
                    },
                    value: None
                }],
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: None,
                representation: TypeRepresentation::Marker(MarkerTypeRepresentation {
                    range: Range::None,
                }),
            })
        );
    }

    #[test]
    fn test_generic_type_definition() {
        assert_eq!(
            Statement::parse("Foo : value => type").unwrap(),
            Statement::TypeDefinition(TypeDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: Some(TypeParameters(vec![TypeParameterName {
                    range: Range::None,
                    value: String::from("value")
                }])),
                representation: TypeRepresentation::Marker(MarkerTypeRepresentation {
                    range: Range::None,
                }),
            })
        );
    }

    #[test]
    fn test_structure_type_definition() {
        let src = r#"Foo : type {
    a :: A
    b :: B
}"#;

        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::TypeDefinition(TypeDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: None,
                representation: TypeRepresentation::Structure(StructureTypeRepresentation {
                    range: Range::None,
                    fields: vec![
                        FieldDefinition {
                            range: Range::None,
                            name: VariableName {
                                range: Range::None,
                                value: String::from("a")
                            },
                            r#type: Type::Named(NamedType {
                                range: Range::None,
                                name: TypeName {
                                    range: Range::None,
                                    value: String::from("A")
                                },
                            }),
                        },
                        FieldDefinition {
                            range: Range::None,
                            name: VariableName {
                                range: Range::None,
                                value: String::from("b")
                            },
                            r#type: Type::Named(NamedType {
                                range: Range::None,
                                name: TypeName {
                                    range: Range::None,
                                    value: String::from("B")
                                },
                            }),
                        },
                    ],
                }),
            })
        );
    }

    #[test]
    fn test_enumeration_type_definition() {
        let src = r#"Foo : type {
    Some Number
    None
}"#;

        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::TypeDefinition(TypeDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: None,
                representation: TypeRepresentation::Enumeration(EnumerationTypeRepresentation {
                    range: Range::None,
                    variants: vec![
                        VariantDefinition {
                            range: Range::None,
                            name: VariantName {
                                range: Range::None,
                                value: String::from("Some")
                            },
                            elements: vec![VariantDefinitionElement(Type::Named(NamedType {
                                range: Range::None,
                                name: TypeName {
                                    range: Range::None,
                                    value: String::from("Number")
                                },
                            }))],
                        },
                        VariantDefinition {
                            range: Range::None,
                            name: VariantName {
                                range: Range::None,
                                value: String::from("None")
                            },
                            elements: Vec::new(),
                        },
                    ],
                }),
            })
        );
    }

    #[test]
    fn test_trait_definition() {
        let src = "Foo : trait Number";
        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::TraitDefinition(TraitDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: None,
                constraints: TraitConstraints {
                    range: Range::None,
                    r#type: Type::Named(NamedType {
                        range: Range::None,
                        name: TypeName {
                            range: Range::None,
                            value: String::from("Number")
                        },
                    }),
                    constraints: None,
                }
            })
        );
    }

    #[test]
    fn test_generic_trait_definition() {
        let src = "Foo : value => trait (value -> Number)";
        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::TraitDefinition(TraitDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: TypeName {
                    range: Range::None,
                    value: String::from("Foo")
                },
                parameters: Some(TypeParameters(vec![TypeParameterName {
                    range: Range::None,
                    value: String::from("value")
                }])),
                constraints: TraitConstraints {
                    range: Range::None,
                    r#type: Type::Function(FunctionType {
                        range: Range::None,
                        inputs: FunctionTypeInputs(vec![Type::Parameter(ParameterType {
                            range: Range::None,
                            name: TypeParameterName {
                                range: Range::None,
                                value: String::from("value")
                            }
                        })]),
                        output: Box::new(Type::Named(NamedType {
                            range: Range::None,
                            name: TypeName {
                                range: Range::None,
                                value: String::from("Number")
                            },
                        })),
                    }),
                    constraints: None,
                }
            })
        );
    }

    #[test]
    fn test_constant_definition() {
        let src = "show :: value -> Unit where (Show value)";
        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::ConstantDefinition(ConstantDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                name: VariableName {
                    range: Range::None,
                    value: String::from("show")
                },
                constraints: ConstantConstraints {
                    range: Range::None,
                    r#type: Type::Function(FunctionType {
                        range: Range::None,
                        inputs: FunctionTypeInputs(vec![Type::Parameter(ParameterType {
                            range: Range::None,
                            name: TypeParameterName {
                                range: Range::None,
                                value: String::from("value")
                            }
                        })]),
                        output: Box::new(Type::Named(NamedType {
                            range: Range::None,
                            name: TypeName {
                                range: Range::None,
                                value: String::from("Unit")
                            },
                        })),
                    }),
                    constraints: Some(Constraints(vec![Constraint::Bound(BoundConstraint {
                        range: Range::None,
                        r#trait: TypeName {
                            range: Range::None,
                            value: String::from("Show")
                        },
                        parameters: vec![Type::Parameter(ParameterType {
                            range: Range::None,
                            name: TypeParameterName {
                                range: Range::None,
                                value: String::from("value")
                            }
                        })],
                    })])),
                }
            })
        );
    }

    #[test]
    fn test_simple_valued_instance_definition() {
        let src = "instance (Foo Number) : 3.14";
        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::InstanceDefinition(InstanceDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                constraints: InstanceConstraints {
                    range: Range::None,
                    bound: BoundConstraint {
                        range: Range::None,
                        r#trait: TypeName {
                            range: Range::None,
                            value: String::from("Foo")
                        },
                        parameters: vec![Type::Named(NamedType {
                            range: Range::None,
                            name: TypeName {
                                range: Range::None,
                                value: String::from("Number")
                            }
                        })],
                    },
                    constraints: None,
                },
                value: Expression::Number(NumberExpression {
                    range: Range::None,
                    value: Number {
                        range: Range::None,
                        value: String::from("3.14")
                    }
                }),
            })
        );
    }

    #[test]
    fn test_complex_valued_instance_definition() {
        let src = "instance (Foo (Maybe value)) where (Foo value) : 3.14";
        assert_eq!(
            Statement::parse(src).unwrap(),
            Statement::InstanceDefinition(InstanceDefinitionStatement {
                range: Range::None,
                comments: Vec::new(),
                attributes: Vec::new(),
                constraints: InstanceConstraints {
                    range: Range::None,
                    bound: BoundConstraint {
                        range: Range::None,
                        r#trait: TypeName {
                            range: Range::None,
                            value: String::from("Foo")
                        },
                        parameters: vec![Type::Parameterized(ParameterizedType {
                            range: Range::None,
                            name: TypeName {
                                range: Range::None,
                                value: String::from("Maybe")
                            },
                            parameters: vec![ParameterizedTypeElement(Type::Parameter(
                                ParameterType {
                                    range: Range::None,
                                    name: TypeParameterName {
                                        range: Range::None,
                                        value: String::from("value")
                                    }
                                }
                            ))],
                        })],
                    },
                    constraints: Some(Constraints(vec![Constraint::Bound(BoundConstraint {
                        range: Range::None,
                        r#trait: TypeName {
                            range: Range::None,
                            value: String::from("Foo")
                        },
                        parameters: vec![Type::Parameter(ParameterType {
                            range: Range::None,
                            name: TypeParameterName {
                                range: Range::None,
                                value: String::from("value")
                            }
                        })],
                    })])),
                },
                value: Expression::Number(NumberExpression {
                    range: Range::None,
                    value: Number {
                        range: Range::None,
                        value: String::from("3.14")
                    }
                }),
            })
        );
    }
}
