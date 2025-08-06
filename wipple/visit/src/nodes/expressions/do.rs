use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{DoExpression, Range};
// TODO

impl Visit for DoExpression {
    fn name(&self) -> &'static str {
        "do"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
