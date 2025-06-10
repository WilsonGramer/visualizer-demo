use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitPattern;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

rule! {
    /// A unit pattern.
    unit_pattern: Typed;
}

impl Visit for UnitPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, _id| {
            (
                ConstraintNode {
                    value: visitor.parent(),
                    constraints: vec![Constraint::Ty(Ty::unit())],
                },
                rule::unit_pattern,
            )
        })
    }
}
