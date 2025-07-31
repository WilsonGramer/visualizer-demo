use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{NumberPattern, Range};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for NumberPattern {
    fn rule(&self) -> Rule {
        "number pattern".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

    fn is_typed(&self) -> bool {
        true
    }
}
