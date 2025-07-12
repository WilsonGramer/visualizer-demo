use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::VariableExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static VARIABLE_NAME: Rule = Rule::new("variable name");
pub static RESOLVED_VARIABLE_NAME: Rule = Rule::new("resolved variable name");
pub static RESOLVED_CONSTANT_NAME: Rule = Rule::new("resolved constant name");
pub static UNRESOLVED_VARIABLE_NAME: Rule = Rule::new("unresolved variable name");

impl Visit for VariableExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            visitor
                .resolve_name(&self.variable.value, id, |definition| match definition {
                    Definition::Variable(definition) => Some((
                        AnnotateNode {
                            value: id,
                            definition: Annotation::Node(definition.node),
                        }
                        .boxed(),
                        RESOLVED_VARIABLE_NAME,
                    )),
                    Definition::Constant(definition) => Some((
                        AnnotateNode {
                            value: id,
                            definition: Annotation::Constant(definition.node),
                        }
                        .boxed(),
                        RESOLVED_CONSTANT_NAME,
                    )),
                    _ => None,
                })
                .unwrap_or_else(|| (EmptyNode.boxed(), UNRESOLVED_VARIABLE_NAME))
        })
    }
}
