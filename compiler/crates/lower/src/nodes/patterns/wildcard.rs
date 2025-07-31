use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, WildcardPattern};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for WildcardPattern {
    fn rule(&self) -> Rule {
        "wildcard pattern".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}
