use crate::{Visit, Visitor};
use wipple_compiler_syntax::VariantPattern;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for VariantPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        todo!()
    }
}
