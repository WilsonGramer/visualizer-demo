use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{DefaultConstraint, Range};
use wipple_compiler_trace::NodeId;
// TODO

impl Visit for DefaultConstraint {
    fn name(&self) -> &'static str {
        "defaultConstraint"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
