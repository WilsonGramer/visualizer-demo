use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{DestructurePattern, Range};
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
