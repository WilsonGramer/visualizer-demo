use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotateExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

pub const ANNOTATED_VALUE: Rule = Rule::new("annotated value");

pub const TYPE_IN_ANNOTATED_VALUE: Rule = Rule::new("type in annotated value");

impl Visit for AnnotateExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let value = self.left.visit(visitor, Some((id, ANNOTATED_VALUE)));

            let ty = self
                .right
                .visit(visitor, Some((value, TYPE_IN_ANNOTATED_VALUE)));

            (
                ConstraintNode {
                    value,
                    constraints: vec![Constraint::Ty(Ty::Of(ty))],
                },
                ANNOTATED_VALUE,
            )
        })
    }
}
