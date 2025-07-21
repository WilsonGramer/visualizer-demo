use crate::{
    Definition, InstanceDefinition, Visit, Visitor,
    attributes::{AttributeParser, InstanceAttributes},
};
use std::{collections::BTreeMap, mem};
use wipple_compiler_syntax::{Constraints, InstanceDefinitionStatement};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static INSTANCE_DEFINITION: Rule = Rule::new("instance definition");
pub static TRAIT_IN_INSTANCE_DEFINITION: Rule = Rule::new("trait in instance definition");
pub static PARAMETER_IN_INSTANCE_DEFINITION: Rule = Rule::new("parameter in instance definition");
pub static CONSTRAINT_IN_INSTANCE_DEFINITION: Rule = Rule::new("constraint in instance definition");
pub static VALUE_IN_INSTANCE_DEFINITION: Rule = Rule::new("value in instance definition");
pub static UNRESOLVED_TRAIT_NAME: Rule = Rule::new("unresolved trait name in instance definition");

impl Visit for InstanceDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let attributes =
                InstanceAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let Some(((trait_node, trait_parameters), _)) =
                visitor.resolve_name(&self.constraints.bound.r#trait.value, id, |definition| {
                    match definition {
                        Definition::Trait(definition) => Some((
                            (definition.node, definition.parameters.clone()),
                            TRAIT_IN_INSTANCE_DEFINITION,
                        )),
                        _ => None,
                    }
                })
            else {
                return (EmptyNode.boxed(), UNRESOLVED_TRAIT_NAME);
            };

            let (value, annotations, substitutions) = visitor.with_definition(|visitor| {
                visitor.current_definition().implicit_type_parameters = true;

                let parameters = self
                    .constraints
                    .bound
                    .parameters
                    .iter()
                    .map(|ty| ty.visit(visitor, (id, PARAMETER_IN_INSTANCE_DEFINITION)));

                // TODO: Ensure `parameters` has the right length
                let substitutions = trait_parameters
                    .into_iter()
                    .zip(parameters)
                    .collect::<BTreeMap<_, _>>();

                let value = visitor.node(
                    (id, VALUE_IN_INSTANCE_DEFINITION),
                    self.value.range(),
                    |visitor, _annotation_id| {
                        let value = self
                            .value
                            .visit(visitor, (_annotation_id, VALUE_IN_INSTANCE_DEFINITION));

                        (
                            AnnotateNode {
                                value,
                                annotations: vec![Annotation::Node(id)],
                            },
                            VALUE_IN_INSTANCE_DEFINITION,
                        )
                    },
                );

                visitor
                    .current_definition()
                    .annotations
                    .push(Annotation::Instantiate {
                        definition: trait_node,
                        substitutions: Some(substitutions.clone()),
                    });

                visitor.current_definition().instantiate_type_parameters = true;

                if let Some(Constraints(constraints)) = &self.constraints.constraints {
                    for constraint in constraints {
                        constraint.visit(visitor, (id, CONSTRAINT_IN_INSTANCE_DEFINITION));
                    }
                }

                (
                    value,
                    mem::take(&mut visitor.current_definition().annotations),
                    substitutions,
                )
            });

            visitor.define_instance(InstanceDefinition {
                node: id,
                comments: self.comments.clone(),
                attributes,
                tr: trait_node,
                substitutions: substitutions.clone(),
                annotations: annotations.clone(),
                value,
            });

            (
                AnnotateNode {
                    value: id,
                    annotations,
                }
                .boxed(),
                INSTANCE_DEFINITION,
            )
        })
    }
}
