use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{AsExpression, Range};
use wipple_visualizer_typecheck::NodeId;
// TODO

impl Visit for AsExpression {
    fn name(&self) -> &'static str {
        "as"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
