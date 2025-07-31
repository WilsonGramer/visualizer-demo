use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, VariantPattern};
use wipple_compiler_trace::{NodeId, Rule};
// TODO

impl Visit for VariantPattern {
    fn rule(&self) -> Rule {
        "variant pattern".into()
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
