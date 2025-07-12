use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NamedType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static RESOLVED_NAMED_TYPE: Rule = Rule::new("resolved named type");
pub static UNRESOLVED_NAMED_TYPE: Rule = Rule::new("unresolved named type");

impl Visit for NamedType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let Some((type_node, rule)) =
                visitor.resolve_name(&self.name.value, id, |definition| match definition {
                    Definition::Type(definition) => Some((definition.node, RESOLVED_NAMED_TYPE)),
                    _ => None,
                })
            else {
                return (EmptyNode.boxed(), UNRESOLVED_NAMED_TYPE);
            };

            // TODO: Ensure `definition.parameters` is empty

            (
                AnnotateNode {
                    value: id,
                    definition: Annotation::Type(type_node, Vec::new()),
                }
                .boxed(),
                rule,
            )
        })
    }
}
