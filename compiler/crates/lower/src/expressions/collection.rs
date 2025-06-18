use crate::{Visit, Visitor};
use wipple_compiler_syntax::CollectionExpression;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for CollectionExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
