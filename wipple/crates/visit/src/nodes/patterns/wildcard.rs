use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{Range, WildcardPattern};

impl Visit for WildcardPattern {
    fn name(&self) -> &'static str {
        "wildcardPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
