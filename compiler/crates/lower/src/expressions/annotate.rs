use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotateExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static ANNOTATED_VALUE: Rule = Rule::new("annotated value");
pub static TYPE_IN_ANNOTATED_VALUE: Rule = Rule::new("type in annotated value");

impl Visit for AnnotateExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let value = self.left.visit(visitor, (id, ANNOTATED_VALUE));
            let ty = self.right.visit(visitor, (value, TYPE_IN_ANNOTATED_VALUE));

            (
                AnnotateNode {
                    value,
                    definition: Annotation::Node(ty),
                },
                ANNOTATED_VALUE,
            )
        })
    }
}
