use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{OrPattern, Range};
// TODO

impl Visit for OrPattern {
    fn name(&self) -> &'static str {
        "orPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
