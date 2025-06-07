use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotateExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

rule! {
    /// A value annotated with a type.
    annotated_value;

    /// A type annotating a value.
    type_in_annotated_value;
}

impl Visit for AnnotateExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let value = self.left.visit(visitor, Some((id, rule::annotated_value)));

            let ty = self
                .right
                .visit(visitor, Some((value, rule::type_in_annotated_value)));

            (
                ConstraintNode {
                    value,
                    constraints: vec![Constraint::Ty(Ty::influenced_by(ty))],
                },
                rule::annotated_value,
            )
        })
    }
}
