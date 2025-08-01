use crate::{
    constraints::constraints_for_call,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{BinaryExpression, Range};
use wipple_compiler_trace::NodeId;

impl Visit for BinaryExpression {
    fn name(&self) -> &'static str {
        "binary"
    }

    fn range(&self) -> Range {
        self.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        match self {
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
                let input = visitor.child(expression.left.as_ref(), id, "inputInApply");
                let function = visitor.child(expression.right.as_ref(), id, "functionInApply");
                visitor.constraints(constraints_for_call(function, [input], id));
            }
        }
    }

    fn is_typed(&self) -> bool {
        true
    }
}
