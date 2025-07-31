use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{EmptyStatement, Range};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for EmptyStatement {
    fn rule(&self) -> Rule {
        "empty statement".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {
        panic!("empty statements should be filtered out")
    }
}
