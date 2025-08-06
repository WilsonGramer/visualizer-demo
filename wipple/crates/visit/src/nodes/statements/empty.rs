use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{EmptyStatement, Range};

impl Visit for EmptyStatement {
    fn name(&self) -> &'static str {
        "emptyStatement"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {
        panic!("empty statements should be filtered out")
    }
}
