use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{PlaceholderType, Range};

impl Visit for PlaceholderType {
    fn name(&self) -> &'static str {
        "placeholderType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
