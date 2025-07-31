use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{InferConstraint, Range};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for InferConstraint {
    fn rule(&self) -> Rule {
        "infer constraint".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
