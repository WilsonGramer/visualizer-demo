use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{Range, TupleType};
use wipple_compiler_typecheck::util::NodeId;
// TODO

impl Visit for TupleType {
    fn name(&self) -> &'static str {
        "tupleType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }
}
