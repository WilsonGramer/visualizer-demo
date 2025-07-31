use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{DefaultConstraint, Range};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for DefaultConstraint {
    fn rule(&self) -> Rule {
        "default constraint".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
