use crate::{
    NeverParenthesized, Parse, Range, Rule, pest_enum,
    tokens::{TypeName, TypeParameterName},
    types::Type,
};
use pest_ast::FromPest;

pest_enum! {
    #[parenthesized = NeverParenthesized<Self>]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Constraint {
        Bound(BoundConstraint),
        Infer(InferConstraint),
        Default(DefaultConstraint),
    }
}

impl Parse for Constraint {
    const RULE: crate::Rule = Rule::constraint;
}

/// ```wipple
/// (Foo value)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::bound_constraint))]
pub struct BoundConstraint {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub r#trait: TypeName,
    pub parameters: Vec<Type>,
}

impl Parse for BoundConstraint {
    const RULE: crate::Rule = Rule::bound_constraint;
}

/// ```wipple
/// (infer value)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::infer_constraint))]
pub struct InferConstraint {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub parameter: TypeParameterName,
}

impl Parse for InferConstraint {
    const RULE: crate::Rule = Rule::infer_constraint;
}

/// ```wipple
/// (value : Number)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::default_constraint))]
pub struct DefaultConstraint {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub parameter: TypeParameterName,
    pub value: Type,
}

impl Parse for DefaultConstraint {
    const RULE: crate::Rule = Rule::default_constraint;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_bound_constraint() {
        assert_eq!(
            Constraint::parse("(Foo value)").unwrap(),
            Constraint::Bound(BoundConstraint {
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
            })
        );
    }

    #[test]
    fn test_infer_constraint() {
        assert_eq!(
            Constraint::parse("(infer value)").unwrap(),
            Constraint::Infer(InferConstraint {
                range: Range::None,
                parameter: TypeParameterName {
                    range: Range::None,
                    value: String::from("value")
                },
            })
        );
    }

    #[test]
    fn test_default_constraint() {
        assert_eq!(
            Constraint::parse("(value : Number)").unwrap(),
            Constraint::Default(DefaultConstraint {
                range: Range::None,
                parameter: TypeParameterName {
                    range: Range::None,
                    value: String::from("value")
                },
                value: Type::Named(NamedType {
                    range: Range::None,
                    name: TypeName {
                        range: Range::None,
                        value: String::from("Number")
                    },
                }),
            })
        );
    }
}
