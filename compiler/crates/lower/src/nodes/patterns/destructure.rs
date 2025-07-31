use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{DestructurePattern, Range};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for DestructurePattern {
    fn rule(&self) -> Rule {
        "destructure pattern".into()
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
