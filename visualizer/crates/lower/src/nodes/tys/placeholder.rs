use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{PlaceholderType, Range};
use wipple_visualizer_typecheck::NodeId;

impl Visit for PlaceholderType {
    fn name(&self) -> &'static str {
        "placeholderType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
