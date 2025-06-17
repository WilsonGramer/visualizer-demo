use crate::Visitor;
use wipple_compiler_syntax::{Attribute, AttributeValue};
use wipple_compiler_trace::Rule;

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

/// An unknown attribute.
pub const UNKNOWN_ATTRIBUTE: Rule = Rule::new("unknown_attribute");

/// A duplicate attribute.
pub const DUPLICATE_ATTRIBUTE: Rule = Rule::new("duplicate_attribute");

/// The attribute is missing a value.
pub const MISSING_ATTRIBUTE_VALUE: Rule = Rule::new("missing_attribute_value");

/// The attribute shouldn't have a value.
pub const EXTRA_ATTRIBUTE_VALUE: Rule = Rule::new("extra_attribute_value");

/// The attribute value is a different type than required.
pub const MISMATCHED_ATTRIBUTE_VALUE: Rule = Rule::new("mismatched_attribute_value");

pub struct AttributeParser<'a, 'v> {
    visitor: &'v mut Visitor<'a>,
    attributes: &'a [Attribute],
}

impl<'a, 'v> AttributeParser<'a, 'v> {
    pub fn new(visitor: &'v mut Visitor<'a>, attributes: &'a [Attribute]) -> Self {
        Self {
            visitor,
            attributes,
        }
    }

    fn parse_name(&mut self, name: &str) -> bool {
        let mut found = false;
        for attribute in self.attributes {
            match attribute {
                Attribute::Name(attribute) => {
                    if attribute.name.source == name {
                        if found {
                            self.visitor
                                .root_placeholder_node(&attribute.range, DUPLICATE_ATTRIBUTE);

                            continue;
                        }

                        found = true;
                    }
                }
                Attribute::Assign(attribute) => {
                    if attribute.name.source == name {
                        self.visitor
                            .root_placeholder_node(&attribute.range, EXTRA_ATTRIBUTE_VALUE);
                    }
                }
            }
        }

        found
    }

    fn parse_text(&mut self, name: &str) -> Option<String> {
        self.parse_assign(name, |value| match value {
            AttributeValue::Text(text) => Some(text.source.clone()),
            #[expect(unreachable_patterns)]
            _ => None,
        })
    }

    fn parse_assign<T>(
        &mut self,
        name: &str,
        f: impl Fn(&'a AttributeValue) -> Option<T>,
    ) -> Option<T> {
        let mut value = None;
        for attribute in self.attributes {
            match attribute {
                Attribute::Name(attribute) => {
                    if attribute.name.source == name {
                        self.visitor
                            .root_placeholder_node(&attribute.range, MISSING_ATTRIBUTE_VALUE);
                    }
                }
                Attribute::Assign(attribute) => {
                    if attribute.name.source == name {
                        if value.is_some() {
                            self.visitor
                                .root_placeholder_node(&attribute.range, DUPLICATE_ATTRIBUTE);

                            continue;
                        }

                        value = f(&attribute.value);

                        if value.is_none() {
                            self.visitor.root_placeholder_node(
                                &attribute.range,
                                MISMATCHED_ATTRIBUTE_VALUE,
                            );
                        }
                    }
                }
            }
        }

        value
    }
}
