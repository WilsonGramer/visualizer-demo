use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{Range, TupleExpression};
// TODO

impl Visit for TupleExpression {
    fn name(&self) -> &'static str {
        "tuple"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
