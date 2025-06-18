use crate::{Visit, Visitor};
use wipple_compiler_syntax::AnnotateExpression;
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

/// A value annotated with a type.
pub const ANNOTATED_VALUE: Rule = Rule::new("annotated_value", &[RuleCategory::Expression]);

/// A type annotating a value.
pub const TYPE_IN_ANNOTATED_VALUE: Rule = Rule::new("type_in_annotated_value", &[]);

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
