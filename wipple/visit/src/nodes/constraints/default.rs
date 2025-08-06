use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{DefaultConstraint, Range};
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
