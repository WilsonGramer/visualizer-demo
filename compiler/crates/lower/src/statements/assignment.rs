use crate::{Visit, Visitor};
use wipple_compiler_syntax::AssignmentStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static ASSIGNMENT_VALUE: Rule = Rule::new("assignment_value");
pub static ASSIGNMENT_PATTERN: Rule = Rule::new("assignment_pattern");
pub static ASSIGNMENT: Rule = Rule::new("assignment");

impl Visit for AssignmentStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let value = self.value.visit(visitor, (id, ASSIGNMENT_VALUE));
            let pattern = self.pattern.visit(visitor, (value, ASSIGNMENT_PATTERN));

            (
                AnnotateNode {
                    value,
                    definition: Annotation::Node(pattern),
                },
                ASSIGNMENT,
            )
        })
    }
}
