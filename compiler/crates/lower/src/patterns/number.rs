use crate::{Visit, Visitor};
use wipple_compiler_syntax::NumberPattern;
use wipple_compiler_trace::{NodeId, Rule};


    // TODO


impl Visit for NumberPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
