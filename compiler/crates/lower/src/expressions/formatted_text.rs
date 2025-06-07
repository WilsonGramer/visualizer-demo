use crate::{Visit, Visitor};
use wipple_compiler_syntax::FormattedTextExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};

rule! {
    // TODO
}

impl Visit for FormattedTextExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        todo!()
    }
}
