use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{AsExpression, Range};
use wipple_compiler_typecheck::util::NodeId;
// TODO

impl Visit for AsExpression {
    fn name(&self) -> &'static str {
        "as"
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
