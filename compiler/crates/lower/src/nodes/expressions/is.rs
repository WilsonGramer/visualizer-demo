use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{IsExpression, Range};
use wipple_visualizer_typecheck::NodeId;
// TODO

impl Visit for IsExpression {
    fn name(&self) -> &'static str {
        "is"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
