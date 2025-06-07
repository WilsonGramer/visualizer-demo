use crate::{Visit, Visitor};
use wipple_compiler_syntax::ExpressionStatement;
use wipple_compiler_trace::{NodeId, Rule, rule};

rule! {}

impl Visit for ExpressionStatement {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        self.expression.visit(visitor, parent)
    }
}
