use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitPattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::UnitNode;

pub static UNIT_PATTERN: Rule = Rule::new("unit pattern");

impl Visit for UnitPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |_visitor, _id| {
            (UnitNode {}, UNIT_PATTERN)
        })
    }
}
