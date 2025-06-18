use crate::{Visit, Visitor};
use wipple_compiler_syntax::SetPattern;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for SetPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
