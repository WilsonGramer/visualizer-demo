use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use wipple_syntax::{ExpressionStatement, Range};

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
