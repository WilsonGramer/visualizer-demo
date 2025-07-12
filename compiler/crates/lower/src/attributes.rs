use crate::Visitor;
use wipple_compiler_syntax::{Attribute, AttributeValue};
use wipple_compiler_trace::{NodeId, Rule};

#[derive(Debug, Clone, Default)]
pub struct ConstantAttributes {
    pub unit: bool,
}

impl ConstantAttributes {
    pub fn parse(parser: &mut AttributeParser<'_, '_>) -> Self {
        ConstantAttributes {
            unit: parser.parse_name("unit"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeAttributes {}

impl TypeAttributes {
    pub fn parse(parser: &mut AttributeParser<'_, '_>) -> Self {
        TypeAttributes {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct TraitAttributes {}

impl TraitAttributes {
    pub fn parse(parser: &mut AttributeParser<'_, '_>) -> Self {
        TraitAttributes {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstanceAttributes {
    pub default: bool,
    pub error: bool,
}

impl InstanceAttributes {
    pub fn parse(parser: &mut AttributeParser<'_, '_>) -> Self {
        InstanceAttributes {
            default: parser.parse_name("default"),
            error: parser.parse_name("error"),
        }
    }
}

pub static UNKNOWN_ATTRIBUTE: Rule = Rule::new("unknown attribute");
pub static DUPLICATE_ATTRIBUTE: Rule = Rule::new("duplicate attribute");
pub static MISSING_ATTRIBUTE_VALUE: Rule = Rule::new("missing attribute value");
pub static EXTRA_ATTRIBUTE_VALUE: Rule = Rule::new("extra attribute value");
pub static MISMATCHED_ATTRIBUTE_VALUE: Rule = Rule::new("mismatched attribute value");

pub struct AttributeParser<'a, 'v> {
    id: NodeId,
    visitor: &'v mut Visitor<'a>,
    attributes: &'a [Attribute],
}

impl<'a, 'v> AttributeParser<'a, 'v> {
    pub fn new(id: NodeId, visitor: &'v mut Visitor<'a>, attributes: &'a [Attribute]) -> Self {
        Self {
            id,
            visitor,
            attributes,
        }
    }

    fn parse_name(&mut self, name: &str) -> bool {
        let mut found = false;
        for attribute in self.attributes {
            if attribute.name.value == name {
                if attribute.value.is_some() {
                    self.visitor
                        .placeholder_node((self.id, EXTRA_ATTRIBUTE_VALUE), attribute.range);
                } else {
                    if found {
                        self.visitor
                            .placeholder_node((self.id, DUPLICATE_ATTRIBUTE), attribute.range);

                        continue;
                    }

                    found = true;
                }
            }
        }

        found
    }

    fn parse_text(&mut self, name: &str) -> Option<String> {
        self.parse_assign(name, |value| match value {
            AttributeValue::Text(text) => Some(text.value.value.clone()),
            #[expect(unreachable_patterns)]
            _ => None,
        })
    }

    fn parse_assign<T>(
        &mut self,
        name: &str,
        f: impl Fn(&'a AttributeValue) -> Option<T>,
    ) -> Option<T> {
        let mut result = None;
        for attribute in self.attributes {
            if attribute.name.value == name {
                if let Some(value) = &attribute.value {
                    if result.is_some() {
                        self.visitor
                            .placeholder_node((self.id, DUPLICATE_ATTRIBUTE), attribute.range);

                        continue;
                    }

                    result = f(value);

                    if result.is_none() {
                        self.visitor.placeholder_node(
                            (self.id, MISMATCHED_ATTRIBUTE_VALUE),
                            attribute.range,
                        );
                    }
                } else {
                    self.visitor
                        .placeholder_node((self.id, MISSING_ATTRIBUTE_VALUE), attribute.range);
                }
            }
        }

        result
    }
}
