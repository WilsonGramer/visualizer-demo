use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

/// A unit expression.
pub const UNIT: Rule = Rule::new("unit");

impl Visit for UnitExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |_visitor, id| {
            (
                ConstraintNode {
                    value: id,
                    constraints: vec![Constraint::Ty(Ty::unit())],
                },
                UNIT,
            )
        })
    }
}
