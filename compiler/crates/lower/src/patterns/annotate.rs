use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotatePattern;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

rule! {
    /// A pattern annotated with a type.
    annotated_pattern;

    /// A type annotating a pattern.
    type_in_annotated_pattern;
}

impl Visit for AnnotatePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let pattern = self
                .left
                .visit(visitor, Some((visitor.parent(), rule::annotated_pattern)));

            let ty = self
                .right
                .visit(visitor, Some((id, rule::type_in_annotated_pattern)));

            (
                ConstraintNode {
                    value: pattern,
                    constraints: vec![Constraint::Ty(Ty::influenced_by(ty))],
                },
                rule::annotated_pattern,
            )
        })
    }
}
