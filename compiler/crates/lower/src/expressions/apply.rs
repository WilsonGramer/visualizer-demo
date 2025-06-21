use crate::{Visit, Visitor};
use wipple_compiler_syntax::ApplyExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::CallNode;

pub const APPLY: Rule = Rule::new("apply");

pub const FUNCTION_IN_APPLY: Rule = Rule::new("function in apply");

pub const INPUT_IN_APPLY: Rule = Rule::new("input in apply");

impl Visit for ApplyExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            let input = self.left.visit(visitor, (id, INPUT_IN_APPLY));

            let function = self.right.visit(visitor, (id, FUNCTION_IN_APPLY));

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
