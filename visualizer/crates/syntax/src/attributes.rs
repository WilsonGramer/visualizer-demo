use crate::{AttributeName, NeverParenthesized, Parse, Range, Rule, Text, pest_enum};
use pest_ast::FromPest;

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::attribute))]
pub struct Attribute {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub name: AttributeName,
    pub value: Option<AttributeValue>,
}

impl Parse for Attribute {
    const RULE: crate::Rule = Rule::attribute;
}

pest_enum! {
    #[parenthesized = NeverParenthesized<Self>]
    #[derive(Debug, Clone, PartialEq)]
    pub enum AttributeValue {
        Text(TextAttributeValue),
    }
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::text_attribute_value))]
pub struct TextAttributeValue {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub value: Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_named_attribute() {
        assert_eq!(
            Attribute::parse("[foo]").unwrap(),
            Attribute {
                range: Range::None,
                name: AttributeName {
                    range: Range::None,
                    value: String::from("foo")
                },
                value: None
            }
        );
    }

    #[test]
    fn test_valued_attribute() {
        assert_eq!(
            Attribute::parse(r#"[a : "b"]"#).unwrap(),
            Attribute {
                range: Range::None,
                name: AttributeName {
                    range: Range::None,
                    value: String::from("a")
                },
                value: Some(AttributeValue::Text(TextAttributeValue {
                    range: Range::None,
                    value: Text {
                        range: Range::None,
                        value: String::from("b")
                    }
                }))
            }
        );
    }
}
