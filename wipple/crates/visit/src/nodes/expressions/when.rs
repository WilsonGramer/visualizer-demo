use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{Range, WhenExpression};
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
