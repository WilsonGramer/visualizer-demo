use crate::{Visit, Visitor};
use wipple_compiler_syntax::AssignmentStatement;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

rule! {
    /// The value is assigned to a pattern.
    assignment_value: Extra;

    /// The pattern in an assignment.
    assignment_pattern: Typed;

    /// An assignment.
    assignment: Extra;
}

impl Visit for AssignmentStatement {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let value = self
                .value
                .visit(visitor, Some((id, rule::assignment_value)));

            // The typechecker doesn't need to see the pattern, since visiting
            // it here will add the relevant constraints
            self.pattern
                .visit(visitor, Some((value, rule::assignment_pattern)));

            (PlaceholderNode, rule::assignment)
        })
    }
}
