use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{IntrinsicExpression, Range};
// TODO

impl Visit for IntrinsicExpression {
    fn name(&self) -> &'static str {
        "intrinsic"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
