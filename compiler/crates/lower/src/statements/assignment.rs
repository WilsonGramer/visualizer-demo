use crate::{Visit, Visitor};
use wipple_compiler_syntax::AssignmentStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

pub const ASSIGNMENT_VALUE: Rule = Rule::new("assignment_value");

pub const ASSIGNMENT_PATTERN: Rule = Rule::new("assignment_pattern");

pub const ASSIGNMENT: Rule = Rule::new("assignment");

impl Visit for AssignmentStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let value = self.value.visit(visitor, (id, ASSIGNMENT_VALUE));

            // The typechecker doesn't need to see the pattern, since visiting
            // it here will add the relevant constraints
            visitor.with_target(value, |visitor| {
                self.pattern.visit(visitor, (value, ASSIGNMENT_PATTERN));
            });

            (PlaceholderNode, ASSIGNMENT)
        })
    }
}
