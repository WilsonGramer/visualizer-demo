use crate::{Visit, Visitor};
use wipple_compiler_syntax::BinaryExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::CallNode;

pub static APPLY: Rule = Rule::new("apply");
pub static INPUT_IN_APPLY: Rule = Rule::new("input in apply");
pub static FUNCTION_IN_APPLY: Rule = Rule::new("function in apply");

impl Visit for BinaryExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range(), |visitor, id| match self {
            BinaryExpression::To(expression) => todo!(),
            BinaryExpression::By(expression) => todo!(),
            BinaryExpression::Power(expression) => todo!(),
            BinaryExpression::Multiply(expression) => todo!(),
            BinaryExpression::Divide(expression) => todo!(),
            BinaryExpression::Remainder(expression) => todo!(),
            BinaryExpression::Add(expression) => todo!(),
            BinaryExpression::Subtract(expression) => todo!(),
            BinaryExpression::LessThan(expression) => todo!(),
            BinaryExpression::LessThanOrEqual(expression) => todo!(),
            BinaryExpression::GreaterThan(expression) => todo!(),
            BinaryExpression::GreaterThanOrEqual(expression) => todo!(),
            BinaryExpression::Equal(expression) => todo!(),
            BinaryExpression::NotEqual(expression) => todo!(),
            BinaryExpression::And(expression) => todo!(),
            BinaryExpression::Or(expression) => todo!(),
            BinaryExpression::Apply(expression) => {
                let input = expression.left.visit(visitor, (id, INPUT_IN_APPLY));

                let function = expression.right.visit(visitor, (id, FUNCTION_IN_APPLY));

                (
                    CallNode {
                        function,
                        inputs: vec![input],
                    },
                    APPLY,
                )
            }
        })
    }
}
