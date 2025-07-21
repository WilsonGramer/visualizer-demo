use crate::{
    Definition, TraitDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TraitAttributes},
};
use std::mem;
use wipple_compiler_syntax::{Constraints, TraitDefinitionStatement};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static TRAIT_DEFINITION: Rule = Rule::new("trait definition");
pub static PARAMETER_IN_TRAIT_DEFINITION: Rule = Rule::new("parameter in trait definition");
pub static TYPE_IN_TRAIT_DEFINITION: Rule = Rule::new("type in trait definition");
pub static CONSTRAINT_IN_TRAIT_DEFINITION: Rule = Rule::new("constraint in trait definition");

impl Visit for TraitDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.name.range, |visitor, id| {
            let attributes =
                TraitAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let parameters = self
                .parameters
                .as_ref()
                .map(|parameters| parameters.0.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|parameter| {
                    let node = visitor
                        .placeholder_node((id, PARAMETER_IN_TRAIT_DEFINITION), parameter.range);

                    visitor.define_name(
                        &parameter.value,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    node
                })
                .collect::<Vec<_>>();

            let (ty, annotations) = visitor.with_definition(|visitor| {
                visitor.current_definition().annotations.extend([
                    Annotation::Instantiate {
                        definition: id,
                        substitutions: None,
                    },
                    Annotation::Bound {
                        node: id,
                        tr: id,
                        substitutions: None,
                    },
                ]);

                // Visiting the type inside of this definition, without
                // `instantiate_type_parameters` set, will keep referenced
                // parameters generic
                let ty = self
                    .constraints
                    .r#type
                    .visit(visitor, (id, TYPE_IN_TRAIT_DEFINITION));

                if let Some(Constraints(constraints)) = &self.constraints.constraints {
                    for constraint in constraints {
                        constraint.visit(visitor, (id, CONSTRAINT_IN_TRAIT_DEFINITION));
                    }
                }

                (ty, mem::take(&mut visitor.current_definition().annotations))
            });

            visitor.define_name(
                &self.name.value,
                Definition::Trait(TraitDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters,
                    annotations,
                }),
            );

            (
                AnnotateNode {
                    value: id,
                    annotations: vec![Annotation::Node(ty)],
                },
                TRAIT_DEFINITION,
            )
        })
    }
}
