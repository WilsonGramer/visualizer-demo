use crate::{Visit, Visitor};
use wipple_compiler_syntax::ApplyExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::CallNode;

rule! {
    /// A function application (using `.`).
    apply;

    /// The function in a function application.
    function_in_apply;

    /// The input in a function application.
    input_in_apply;
}

impl Visit for ApplyExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let input = self.left.visit(visitor, Some((id, rule::input_in_apply)));

            let function = self
                .right
                .visit(visitor, Some((id, rule::function_in_apply)));

            (
                CallNode {
                    function,
                    inputs: vec![input],
                },
                rule::apply,
            )
        })
    }
}
