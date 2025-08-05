use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{ExpressionStatement, Range};
use wipple_visualizer_typecheck::NodeId;

impl Visit for ExpressionStatement {
    fn name(&self) -> &'static str {
        "expressionStatement"
    }

    fn range(&self) -> Range {
        self.expression.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.child(&self.expression, id, "expressionInExpressionStatement");
    }

    fn hide(&self) -> bool {
        true
    }
}
