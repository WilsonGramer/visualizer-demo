use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{InferConstraint, Range};
// TODO

impl Visit for InferConstraint {
    fn name(&self) -> &'static str {
        "inferConstraint"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
