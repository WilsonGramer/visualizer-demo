use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotatePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

pub const ANNOTATED_PATTERN: Rule = Rule::new("annotated pattern");

pub const TYPE_IN_ANNOTATED_PATTERN: Rule = Rule::new("type in annotated pattern");

impl Visit for AnnotatePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let pattern = self.left.visit(visitor, (id, ANNOTATED_PATTERN));

            let ty = visitor.with_target(pattern, |visitor| {
                self.right.visit(visitor, (id, TYPE_IN_ANNOTATED_PATTERN))
            });

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
