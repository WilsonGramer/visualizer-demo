use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{AsExpression, Range};
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
