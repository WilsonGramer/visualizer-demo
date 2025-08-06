use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{Range, TextPattern};
// TODO

impl Visit for TextPattern {
    fn name(&self) -> &'static str {
        "textPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
