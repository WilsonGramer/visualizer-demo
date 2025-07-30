use crate::{Definition, Visit, Visitor};
use std::collections::BTreeMap;
use wipple_compiler_syntax::TextExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static TEXT: Rule = Rule::new("text");
pub static MISSING_TEXT_TYPE: Rule = Rule::new("missing text type");

impl Visit for TextExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let text_ty = visitor.resolve_name("Text", id, |definition| match definition {
                Definition::Type(definition) => Some((definition.node, TEXT)),
                _ => None,
            });

            match text_ty {
                Some((text_ty, rule)) => (
                    AnnotateNode {
                        value: id,
                        annotations: vec![Annotation::Named {
                            definition: text_ty,
                            parameters: BTreeMap::new(),
                        }],
                    }
                    .boxed(),
                    rule,
                ),
                None => (EmptyNode.boxed(), MISSING_TEXT_TYPE),
            }
        })
    }
}
