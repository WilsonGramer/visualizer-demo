use crate::{Visit, Visitor};
use wipple_compiler_syntax::TraitDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};

// TODO

impl Visit for TraitDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        todo!()
    }
}
