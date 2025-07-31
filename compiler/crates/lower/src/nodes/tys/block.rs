use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{BlockType, Range};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for BlockType {
    fn rule(&self) -> Rule {
        "block type".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
