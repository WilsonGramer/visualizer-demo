use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, VariantPattern};
use wipple_compiler_trace::NodeId;
// TODO

impl Visit for VariantPattern {
    fn name(&self) -> &'static str {
        "variantPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
