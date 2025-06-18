use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotatePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

/// A pattern annotated with a type.
pub const ANNOTATED_PATTERN: Rule = Rule::new("annotated_pattern");

/// A type annotating a pattern.
pub const TYPE_IN_ANNOTATED_PATTERN: Rule = Rule::new("type_in_annotated_pattern");

impl Visit for AnnotatePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let pattern = self
                .left
                .visit(visitor, Some((visitor.parent(), ANNOTATED_PATTERN)));

            let ty = self
                .right
                .visit(visitor, Some((id, TYPE_IN_ANNOTATED_PATTERN)));

            (
                ConstraintNode {
                    value: pattern,
                    constraints: vec![Constraint::Ty(Ty::Of(ty))],
                },
                ANNOTATED_PATTERN,
            )
        })
    }
}
