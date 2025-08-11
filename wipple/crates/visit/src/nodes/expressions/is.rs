use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{IsExpression, Range};
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
