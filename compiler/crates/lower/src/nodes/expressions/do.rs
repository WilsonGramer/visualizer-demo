use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{DoExpression, Range};
use wipple_visualizer_typecheck::NodeId;
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
