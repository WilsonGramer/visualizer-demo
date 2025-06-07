use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitType;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

rule! {
    /// The unit type.
    unit_type;
}

impl Visit for UnitType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |_visitor, id| {
            (
                ConstraintNode {
                    value: id,
                    constraints: vec![Constraint::Ty(Ty::unit())],
                },
                rule::unit_type,
            )
        })
    }
}
