use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{PlaceholderExpression, Range};

impl Visit for PlaceholderExpression {
    fn name(&self) -> &'static str {
        "placeholder"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
