use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{Range, StructureExpression};
// TODO

impl Visit for StructureExpression {
    fn name(&self) -> &'static str {
        "structure"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
