use crate::{Visit, Visitor};
use wipple_compiler_syntax::AssignmentStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

/// The value is assigned to a pattern.
pub const ASSIGNMENT_VALUE: Rule = Rule::new("assignment_value", &[]);

/// The pattern in an assignment.
pub const ASSIGNMENT_PATTERN: Rule = Rule::new("assignment_pattern", &[]);

/// An assignment.
pub const ASSIGNMENT: Rule = Rule::new("assignment", &[]);

impl Visit for AssignmentStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let value = self.value.visit(visitor, Some((id, ASSIGNMENT_VALUE)));

            // The typechecker doesn't need to see the pattern, since visiting
            // it here will add the relevant constraints
            self.pattern
                .visit(visitor, Some((value, ASSIGNMENT_PATTERN)));

            (PlaceholderNode, ASSIGNMENT)
        })
    }
}
