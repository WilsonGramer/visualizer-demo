use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, StructureExpression};
use wipple_compiler_typecheck::util::NodeId;
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

    fn is_typed(&self) -> bool {
        true
    }
}
