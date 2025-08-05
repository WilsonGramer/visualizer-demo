use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{NumberPattern, Range};
use wipple_visualizer_typecheck::NodeId;
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
