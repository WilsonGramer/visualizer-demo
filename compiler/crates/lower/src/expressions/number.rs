use crate::{Definition, Visit, Visitor};
use std::collections::BTreeMap;
use wipple_compiler_syntax::NumberExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation, EmptyNode, Node};

pub static NUMBER: Rule = Rule::new("number");
pub static MISSING_NUMBER_TYPE: Rule = Rule::new("missing number type");

impl Visit for NumberExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let number_ty = visitor.resolve_name("Number", id, |definition| match definition {
                Definition::Type(definition) => Some((definition.node, NUMBER)),
                _ => None,
            });

            match number_ty {
                Some((number_ty, rule)) => (
                    AnnotateNode {
                        value: id,
                        annotations: vec![Annotation::Type {
                            definition: number_ty,
                            parameters: BTreeMap::new(),
                        }],
                    }
                    .boxed(),
                    rule,
                ),
                None => (EmptyNode.boxed(), MISSING_NUMBER_TYPE),
            }
        })
    }
}
