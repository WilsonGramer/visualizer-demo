use crate::visitor::Visitor;
use wipple_compiler_syntax::{Attribute, AttributeValue};
use wipple_compiler_trace::{Fact, NodeId};

#[derive(Debug, Clone, Default)]
pub struct ConstantAttributes {
    pub unit: bool,
}

impl ConstantAttributes {
    pub(crate) fn parse(visitor: &mut Visitor<'_>, parser: &mut AttributeParser<'_>) -> Self {
        ConstantAttributes {
            unit: parser.parse_name(visitor, "unit"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeAttributes {}

impl TypeAttributes {
    pub(crate) fn parse(_visitor: &mut Visitor<'_>, parser: &mut AttributeParser<'_>) -> Self {
        TypeAttributes {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct TraitAttributes {}

impl TraitAttributes {
    pub(crate) fn parse(_visitor: &mut Visitor<'_>, parser: &mut AttributeParser<'_>) -> Self {
        TraitAttributes {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstanceAttributes {
    pub default: bool,
    pub error: bool,
}

impl InstanceAttributes {
    pub(crate) fn parse(visitor: &mut Visitor<'_>, parser: &mut AttributeParser<'_>) -> Self {
        InstanceAttributes {
            default: parser.parse_name(visitor, "default"),
            error: parser.parse_name(visitor, "error"),
        }
    }
}

pub(crate) struct AttributeParser<'a> {
    id: NodeId,
    attributes: &'a [Attribute],
}

impl<'a> AttributeParser<'a> {
    pub(crate) fn new(id: NodeId, attributes: &'a [Attribute]) -> Self {
        Self { id, attributes }
    }

    fn parse_name(&mut self, visitor: &mut Visitor<'_>, name: &str) -> bool {
        let mut found = false;
        for attribute in self.attributes {
            if attribute.name.value == name {
                let node = visitor.node(attribute.range, "attribute");

                if attribute.value.is_some() {
                    visitor.fact(node, Fact::marker("extraAttributeValue"));
                } else {
                    if found {
                        visitor.fact(node, Fact::marker("duplicateAttribute"));

                        continue;
                    }

                    found = true;
                }
            }
        }

        found
    }

    fn parse_text(&mut self, visitor: &mut Visitor<'_>, name: &str) -> Option<String> {
        self.parse_assign(visitor, name, |value| match value {
            AttributeValue::Text(text) => Some(text.value.value.clone()),
            #[expect(unreachable_patterns)]
            _ => None,
        })
    }

    fn parse_assign<T>(
        &mut self,
        visitor: &mut Visitor<'_>,
        name: &str,
        f: impl Fn(&'a AttributeValue) -> Option<T>,
    ) -> Option<T> {
        let mut result = None;
        for attribute in self.attributes {
            if attribute.name.value == name {
                let node = visitor.node(attribute.range, "attribute");

                if let Some(value) = &attribute.value {
                    if result.is_some() {
                        visitor.fact(node, Fact::marker("duplicateAttribute"));

                        continue;
                    }

                    result = f(value);

                    if result.is_none() {
                        visitor.fact(node, Fact::marker("mismatchedAttributeValue"));
                    }
                } else {
                    visitor.fact(node, Fact::marker("missingAttributeValue"));
                }
            }
        }

        result
    }
}
