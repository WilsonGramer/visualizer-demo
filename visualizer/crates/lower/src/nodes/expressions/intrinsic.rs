use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{IntrinsicExpression, Range};
use wipple_visualizer_typecheck::NodeId;
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
