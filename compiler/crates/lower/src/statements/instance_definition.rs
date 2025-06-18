use crate::{Visit, Visitor};
use wipple_compiler_syntax::InstanceDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for InstanceDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
