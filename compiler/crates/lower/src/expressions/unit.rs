use crate::{Visit, Visitor};
use wipple_compiler_syntax::UnitExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::UnitNode;

pub static UNIT: Rule = Rule::new("unit");

impl Visit for UnitExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |_visitor, _id| (UnitNode {}, UNIT))
    }
}
