use crate::{
    Definition, TypeDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TypeAttributes},
};
use wipple_compiler_syntax::TypeDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

pub const TYPE_DEFINITION: Rule = Rule::new("type definition");

pub const PARAMETER_IN_TYPE_DEFINITION: Rule = Rule::new("parameter in type definition");

impl Visit for TypeDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.name.range, |visitor, id| {
            let attributes =
                TypeAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let parameters = self
                .parameters
                .as_ref()
                .map(|parameters| parameters.0.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|parameter| {
                    let node = visitor
                        .placeholder_node((id, PARAMETER_IN_TYPE_DEFINITION), parameter.range);

                    visitor.define_name(
                        &parameter.value,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    node
                })
                .collect::<Vec<_>>();

            visitor.define_name(
                &self.name.value,
                Definition::Type(TypeDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters,
                }),
            );

            (PlaceholderNode, TYPE_DEFINITION)
        })
    }
}
