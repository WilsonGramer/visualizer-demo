use crate::{
    Parse, Range, Rule, Type, pest_enum,
    tokens::{Number, Text, VariableName, VariantName},
};
use pest_ast::FromPest;

pest_enum! {
    #[parenthesized = ParenthesizedPattern]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Pattern {
        Unit(UnitPattern),
        Wildcard(WildcardPattern),
        Variable(VariablePattern),
        Number(NumberPattern),
        Text(TextPattern),
        Destructure(DestructurePattern),
        Set(SetPattern),
        Variant(VariantPattern),
        Or(OrPattern),
        Tuple(TuplePattern),
        Annotate(AnnotatePattern),
    }
}

impl Parse for Pattern {
    const RULE: crate::Rule = Rule::pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::parenthesized_pattern))]
pub struct ParenthesizedPattern(pub Pattern);

impl Pattern {
    pub fn range(&self) -> Range {
        match self {
            Pattern::Unit(pattern) => pattern.range,
            Pattern::Wildcard(pattern) => pattern.range,
            Pattern::Variable(pattern) => pattern.range,
            Pattern::Number(pattern) => pattern.range,
            Pattern::Text(pattern) => pattern.range,
            Pattern::Destructure(pattern) => pattern.range,
            Pattern::Set(pattern) => pattern.range,
            Pattern::Variant(pattern) => pattern.range,
            Pattern::Or(pattern) => pattern.range,
            Pattern::Tuple(pattern) => pattern.range,
            Pattern::Annotate(pattern) => pattern.range,
        }
    }
}

/// ```wipple
/// _
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::wildcard_pattern))]
pub struct WildcardPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

impl Parse for WildcardPattern {
    const RULE: crate::Rule = Rule::wildcard_pattern;
}

/// ```wipple
/// x
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variable_pattern))]
pub struct VariablePattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub variable: VariableName,
}

impl Parse for VariablePattern {
    const RULE: crate::Rule = Rule::variable_pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::number_pattern))]
pub struct NumberPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub value: Number,
}

impl Parse for NumberPattern {
    const RULE: crate::Rule = Rule::number_pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::text_pattern))]
pub struct TextPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub value: Text,
}

impl Parse for TextPattern {
    const RULE: crate::Rule = Rule::text_pattern;
}

/// ```wipple
/// {x : y}
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::destructure_pattern))]
pub struct DestructurePattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub fields: Vec<DestructurePatternField>,
}

impl Parse for DestructurePattern {
    const RULE: crate::Rule = Rule::destructure_pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::destructure_pattern_field))]
pub struct DestructurePatternField {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: VariableName,
    pub value: Pattern,
}

impl Parse for DestructurePatternField {
    const RULE: crate::Rule = Rule::destructure_pattern_field;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::unit_pattern))]
pub struct UnitPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
}

impl Parse for UnitPattern {
    const RULE: crate::Rule = Rule::unit_pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::tuple_pattern))]
pub struct TuplePattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub elements: Vec<Pattern>,
}

impl Parse for TuplePattern {
    const RULE: crate::Rule = Rule::tuple_pattern;
}

/// ```wipple
/// x or y or z
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::or_pattern))]
pub struct OrPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub patterns: Vec<Pattern>,
}

impl Parse for OrPattern {
    const RULE: crate::Rule = Rule::or_pattern;
}

/// ```wipple
/// set x
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::set_pattern))]
pub struct SetPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub variable: VariableName,
}

impl Parse for SetPattern {
    const RULE: crate::Rule = Rule::set_pattern;
}

/// ```wipple
/// Some x y z
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variant_pattern))]
pub struct VariantPattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub variant: VariantName,
    pub elements: Vec<VariantPatternElement>,
}

impl Parse for VariantPattern {
    const RULE: crate::Rule = Rule::variant_pattern;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::variant_pattern_element))]
pub struct VariantPatternElement(pub Pattern);

impl Parse for VariantPatternElement {
    const RULE: crate::Rule = Rule::variant_pattern_element;
}

/// ```wipple
/// (x :: Number)
/// ```
#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::annotate_pattern))]
pub struct AnnotatePattern {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub left: Box<Pattern>,
    pub right: Type,
}

impl Parse for AnnotatePattern {
    const RULE: crate::Rule = Rule::annotate_pattern;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_wildcard_pattern() {
        assert_eq!(
            Pattern::parse("_").unwrap(),
            Pattern::Wildcard(WildcardPattern { range: Range::None })
        );
    }

    #[test]
    fn test_variable_pattern() {
        assert_eq!(
            Pattern::parse("x").unwrap(),
            Pattern::Variable(VariablePattern {
                range: Range::None,
                variable: VariableName {
                    range: Range::None,
                    value: String::from("x")
                },
            })
        );
    }

    #[test]
    fn test_destructure_pattern() {
        assert_eq!(
            Pattern::parse("{x : y}").unwrap(),
            Pattern::Destructure(DestructurePattern {
                range: Range::None,
                fields: vec![DestructurePatternField {
                    range: Range::None,
                    name: VariableName {
                        range: Range::None,
                        value: String::from("x")
                    },
                    value: Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        },
                    }),
                },],
            })
        );
    }

    #[test]
    fn test_set_pattern() {
        assert_eq!(
            Pattern::parse("set x").unwrap(),
            Pattern::Set(SetPattern {
                range: Range::None,
                variable: VariableName {
                    range: Range::None,
                    value: String::from("x")
                },
            })
        );
    }

    #[test]
    fn test_simple_variant_pattern() {
        assert_eq!(
            Pattern::parse("None").unwrap_or_else(|e| panic!("{e:#?}")),
            Pattern::Variant(VariantPattern {
                range: Range::None,
                variant: VariantName {
                    range: Range::None,
                    value: String::from("None")
                },
                elements: Vec::new(),
            })
        );
    }

    #[test]
    fn test_complex_variant_pattern() {
        assert_eq!(
            Pattern::parse("Some x y z").unwrap(),
            Pattern::Variant(VariantPattern {
                range: Range::None,
                variant: VariantName {
                    range: Range::None,
                    value: String::from("Some")
                },
                elements: vec![
                    VariantPatternElement(Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        }
                    })),
                    VariantPatternElement(Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        }
                    })),
                    VariantPatternElement(Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("z")
                        }
                    })),
                ],
            })
        );
    }

    #[test]
    fn test_simple_or_pattern() {
        assert_eq!(
            Pattern::parse("x or y").unwrap(),
            Pattern::Or(OrPattern {
                range: Range::None,
                patterns: vec![
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        }
                    }),
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        }
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_complex_or_pattern() {
        assert_eq!(
            Pattern::parse("x or y or z").unwrap(),
            Pattern::Or(OrPattern {
                range: Range::None,
                patterns: vec![
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("x")
                        }
                    }),
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("y")
                        }
                    }),
                    Pattern::Variable(VariablePattern {
                        range: Range::None,
                        variable: VariableName {
                            range: Range::None,
                            value: String::from("z")
                        }
                    }),
                ],
            })
        );
    }
}
