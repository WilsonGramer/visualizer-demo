use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotatePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static ANNOTATED_PATTERN: Rule = Rule::new("annotated pattern");
pub static TYPE_IN_ANNOTATED_PATTERN: Rule = Rule::new("type in annotated pattern");

impl Visit for AnnotatePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let pattern = self.left.visit(visitor, (id, ANNOTATED_PATTERN));

            let ty = self
                .right
                .visit(visitor, (pattern, TYPE_IN_ANNOTATED_PATTERN));

            (
                AnnotateNode {
                    value: pattern,
                    definition: Annotation::Node(ty),
                },
                ANNOTATED_PATTERN,
            )
        })
    }
}
