use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{ExpressionStatement, Range};
use wipple_compiler_typecheck::util::NodeId;

impl Visit for ExpressionStatement {
    fn name(&self) -> &'static str {
        "_expressionStatement"
    }

    fn range(&self) -> Range {
        self.expression.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.child(&self.expression, id, "expressionInExpressionStatement");
    }
}
