use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{BlockType, Range};
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
