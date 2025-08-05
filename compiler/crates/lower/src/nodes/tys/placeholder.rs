use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{PlaceholderType, Range};
use wipple_compiler_typecheck::util::NodeId;

impl Visit for PlaceholderType {
    fn name(&self) -> &'static str {
        "placeholderType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
