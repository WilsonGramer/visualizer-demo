use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{PlaceholderType, Range};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for PlaceholderType {
    fn rule(&self) -> Rule {
        "placeholder type".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
