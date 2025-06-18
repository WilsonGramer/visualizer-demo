use crate::{Visit, Visitor};
use wipple_compiler_syntax::ApplyExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::CallNode;

/// A function application (using `.`).
pub const APPLY: Rule = Rule::new("apply");

/// The function in a function application.
pub const FUNCTION_IN_APPLY: Rule = Rule::new("function_in_apply");

/// The input in a function application.
pub const INPUT_IN_APPLY: Rule = Rule::new("input_in_apply");

impl Visit for ApplyExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let input = self.left.visit(visitor, Some((id, INPUT_IN_APPLY)));

            let function = self.right.visit(visitor, Some((id, FUNCTION_IN_APPLY)));

            (
                CallNode {
                    function,
                    inputs: vec![input],
                },
                APPLY,
            )
        })
    }
}
