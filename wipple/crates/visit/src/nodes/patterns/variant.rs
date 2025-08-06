use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{Range, VariantPattern};
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
