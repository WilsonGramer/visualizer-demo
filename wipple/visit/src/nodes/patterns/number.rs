use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{NumberPattern, Range};
// TODO

impl Visit for NumberPattern {
    fn name(&self) -> &'static str {
        "numberPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
