use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{DestructurePattern, Range};
use wipple_compiler_typecheck::util::NodeId;
// TODO

impl Visit for DestructurePattern {
    fn name(&self) -> &'static str {
        "destructurePattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
