use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{Range, WhenExpression};
use wipple_visualizer_typecheck::NodeId;
// TODO

impl Visit for WhenExpression {
    fn name(&self) -> &'static str {
        "when"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
