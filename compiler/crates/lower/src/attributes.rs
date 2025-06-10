use crate::Visitor;
use wipple_compiler_syntax::{Attribute, AttributeValue};
use wipple_compiler_trace::rule;

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

rule! {
    /// An unknown attribute.
    unknown_attribute: Extra;

    /// A duplicate attribute.
    duplicate_attribute: Extra;

    /// The attribute is missing a value.
    missing_attribute_value: Extra;

    /// The attribute shouldn't have a value.
    extra_attribute_value: Extra;

    /// The attribute value is a different type than required.
    mismatched_attribute_value: Extra;
}

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
                                .root_placeholder_node(&attribute.range, rule::duplicate_attribute);

                            continue;
                        }

                        found = true;
                    }
                }
                Attribute::Assign(attribute) => {
                    if attribute.name.source == name {
                        self.visitor
                            .root_placeholder_node(&attribute.range, rule::extra_attribute_value);
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
                            .root_placeholder_node(&attribute.range, rule::missing_attribute_value);
                    }
                }
                Attribute::Assign(attribute) => {
                    if attribute.name.source == name {
                        if value.is_some() {
                            self.visitor
                                .root_placeholder_node(&attribute.range, rule::duplicate_attribute);

                            continue;
                        }

                        value = f(&attribute.value);

                        if value.is_none() {
                            self.visitor.root_placeholder_node(
                                &attribute.range,
                                rule::mismatched_attribute_value,
                            );
                        }
                    }
                }
            }
        }

        value
    }
}
