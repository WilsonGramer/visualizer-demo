use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{PlaceholderExpression, Range};
use wipple_compiler_typecheck::util::NodeId;

impl Visit for PlaceholderExpression {
    fn name(&self) -> &'static str {
        "placeholder"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}

    fn is_typed(&self) -> bool {
        true
    }
}
