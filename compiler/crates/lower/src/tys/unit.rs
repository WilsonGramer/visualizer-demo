use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

pub const UNIT_TYPE: Rule = Rule::new("unit type");

impl Visit for UnitType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.range, |visitor, _id| {
            (
                ConstraintNode {
                    value: visitor.target(),
                    constraints: vec![Constraint::Ty(Ty::unit())],
                },
                UNIT_TYPE,
            )
        })
    }
}
