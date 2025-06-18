use crate::{Visit, Visitor};
use wipple_compiler_syntax::DoExpression;
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};

// TODO

impl Visit for DoExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
