use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, TupleExpression};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for TupleExpression {
    fn rule(&self) -> Rule {
        "tuple".into()
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
