use crate::{Definition, TypeParameterDefinition, Visit, Visitor};
use wipple_compiler_syntax::ParameterType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static PARAMETER_TYPE: Rule = Rule::new("parameter type");
pub static UNRESOLVED_PARAMETER_TYPE: Rule = Rule::new("unresolved parameter type");

impl Visit for ParameterType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let existing =
                visitor.resolve_name(&self.name.value, id, |definition| match definition {
                    Definition::TypeParameter(definition) => {
                        Some((definition.node, PARAMETER_TYPE))
                    }
                    _ => None,
                });

            match existing {
                Some((node, rule)) => (
                    AnnotateNode {
                        value: id,
                        definition: Annotation::Node(node),
                    }
                    .boxed(),
                    rule,
                ),
                None => {
                    if visitor.implicit_type_parameters {
                        visitor.define_name(
                            &self.name.value,
                            Definition::TypeParameter(TypeParameterDefinition { node: id }),
                        );

                        (EmptyNode.boxed(), PARAMETER_TYPE)
                    } else {
                        (EmptyNode.boxed(), UNRESOLVED_PARAMETER_TYPE)
                    }
                }
            }
        })
    }
}
