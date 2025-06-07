use crate::{Visit, Visitor};
use wipple_compiler_syntax::ConstantDefinitionStatement;
use wipple_compiler_trace::{rule, NodeId, Rule};

rule! {
    // TODO
}

impl Visit for ConstantDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        todo!()
    }
}
