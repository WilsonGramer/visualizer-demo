use crate::{Definition, Visit, Visitor};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{ParameterizedType, ParameterizedTypeElement};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static RESOLVED_PARAMETERIZED_TYPE: Rule = Rule::new("resolved parameterized type");
pub static UNRESOLVED_PARAMETERIZED_TYPE: Rule = Rule::new("unresolved parameterized type");
pub static PARAMETER_IN_PARAMETERIZED_TYPE: Rule = Rule::new("parameter in parameterized type");

impl Visit for ParameterizedType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let Some(((type_node, type_parameters), rule)) =
                visitor.resolve_name(&self.name.value, id, |definition| match definition {
                    Definition::Type(definition) => Some((
                        (definition.node, definition.parameters.clone()),
                        RESOLVED_PARAMETERIZED_TYPE,
                    )),
                    _ => None,
                })
            else {
                return (EmptyNode.boxed(), UNRESOLVED_PARAMETERIZED_TYPE);
            };

            let parameters = self.parameters.iter().map(|ParameterizedTypeElement(ty)| {
                ty.visit(visitor, (id, PARAMETER_IN_PARAMETERIZED_TYPE))
            });

            // TODO: Ensure `parameters` has the right length

            (
                AnnotateNode {
                    value: id,
                    annotations: vec![Annotation::Named {
                        definition: type_node,
                        parameters: type_parameters
                            .into_iter()
                            .zip(parameters)
                            .collect::<BTreeMap<_, _>>(),
                    }],
                }
                .boxed(),
                rule,
            )
        })
    }
}
