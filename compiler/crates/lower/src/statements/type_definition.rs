use crate::{
    Definition, TypeDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TypeAttributes},
};
use std::{collections::BTreeMap, mem};
use wipple_compiler_syntax::TypeDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{Annotation, EmptyNode};

pub static TYPE_DEFINITION: Rule = Rule::new("type definition");
pub static PARAMETER_IN_TYPE_DEFINITION: Rule = Rule::new("parameter in type definition");

impl Visit for TypeDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        // NOT a `definition_node`; type definitions are referenced by ID but
        // not instantiated in this way
        visitor.definition_node(parent, self.name.range, |visitor, id| {
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

            let annotations = visitor.with_definition(|visitor| {
                visitor
                    .current_definition()
                    .annotations
                    .push(Annotation::Named {
                        definition: id,
                        parameters: BTreeMap::new(),
                    });

                // Types don't have additional constraints

                mem::take(&mut visitor.current_definition().annotations)
            });

            visitor.define_name(
                &self.name.value,
                Definition::Type(TypeDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters,
                    annotations,
                }),
            );

            (EmptyNode, TYPE_DEFINITION)
        })
    }
}
