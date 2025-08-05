use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{DoExpression, Range};
use wipple_compiler_typecheck::util::NodeId;
// TODO

impl Visit for DoExpression {
    fn name(&self) -> &'static str {
        "do"
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
