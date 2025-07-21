use crate::{Visit, Visitor};
use wipple_compiler_syntax::InferConstraint;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for InferConstraint {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        todo!()
    }
}
