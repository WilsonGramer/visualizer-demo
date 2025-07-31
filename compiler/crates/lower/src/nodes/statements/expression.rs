use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{ExpressionStatement, Range};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for ExpressionStatement {
    fn rule(&self) -> Rule {
        "expression statement [ignore]".into()
    }

    fn range(&self) -> Range {
        self.expression.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.child(&self.expression, id, "expression in expression statement");
    }
}
