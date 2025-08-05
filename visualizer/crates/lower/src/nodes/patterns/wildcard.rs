use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{Range, WildcardPattern};
use wipple_visualizer_typecheck::NodeId;

impl Visit for WildcardPattern {
    fn name(&self) -> &'static str {
        "wildcardPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
