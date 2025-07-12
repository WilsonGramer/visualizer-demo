use crate::{
    Definition, InstanceDefinition, Visit, Visitor,
    attributes::{AttributeParser, InstanceAttributes},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::InstanceDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static INSTANCE_DEFINITION: Rule = Rule::new("instance definition");
pub static TRAIT_IN_INSTANCE_DEFINITION: Rule = Rule::new("trait in instance definition");
pub static PARAMETER_IN_INSTANCE_DEFINITION: Rule = Rule::new("parameter in instance definition");
pub static VALUE_IN_INSTANCE_DEFINITION: Rule = Rule::new("value in instance definition");
pub static UNRESOLVED_TRAIT_NAME: Rule = Rule::new("unresolved trait name in instance definition");

impl Visit for InstanceDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
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

            let parameters = self
                .constraints
                .bound
                .parameters
                .iter()
                .map(|ty| {
                    visitor.with_implicit_type_parameters(|visitor| {
                        ty.visit(visitor, (id, PARAMETER_IN_INSTANCE_DEFINITION))
                    })
                })
                .collect::<Vec<_>>();

            // TODO
            let constraints = self
                .constraints
                .constraints
                .iter()
                .map(|_| todo!())
                .collect::<Vec<_>>();

            visitor.define_instance(InstanceDefinition {
                node: id,
                comments: self.comments.clone(),
                attributes,
                tr: trait_node,
                parameters: parameters.clone(),
                constraints,
            });

            let value = self
                .value
                .visit(visitor, (id, VALUE_IN_INSTANCE_DEFINITION));

            // TODO: Ensure `parameters` has the right length

            let substitutions = trait_parameters
                .into_iter()
                .zip(parameters)
                .collect::<BTreeMap<_, _>>();

            (
                AnnotateNode {
                    value,
                    definition: Annotation::Trait(trait_node, substitutions),
                }
                .boxed(),
                INSTANCE_DEFINITION,
            )
        })
    }
}
