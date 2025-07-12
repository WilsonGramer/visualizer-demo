use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::TupleNode;

pub static UNIT_TYPE: Rule = Rule::new("unit type");

impl Visit for UnitType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |_visitor, _id| {
            (
                TupleNode {
                    elements: Vec::new(),
                },
                UNIT_TYPE,
            )
        })
    }
}
