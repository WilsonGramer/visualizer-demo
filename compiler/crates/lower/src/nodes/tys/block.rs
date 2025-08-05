use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{BlockType, Range};
use wipple_compiler_typecheck::util::NodeId;
// TODO

impl Visit for BlockType {
    fn name(&self) -> &'static str {
        "blockType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
