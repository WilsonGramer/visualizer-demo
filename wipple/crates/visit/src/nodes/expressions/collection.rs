use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{CollectionExpression, Range};
// TODO

impl Visit for CollectionExpression {
    fn name(&self) -> &'static str {
        "collection"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
