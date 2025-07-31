use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, TupleType};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for TupleType {
    fn rule(&self) -> Rule {
        "tuple type".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
