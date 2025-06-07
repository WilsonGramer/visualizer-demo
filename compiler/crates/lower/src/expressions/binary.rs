use crate::{Visit, Visitor};
use wipple_compiler_syntax::BinaryExpression;
use wipple_compiler_trace::{rule, NodeId, Rule};

rule! {
    // TODO
}

impl Visit for BinaryExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        match self.operator.source.as_str() {
            "to" => visit_as_math_expression(self, visitor, "To"),
            "by" => visit_as_math_expression(self, visitor, "By"),
            "^" => visit_as_math_expression(self, visitor, "Power"),
            "*" => visit_as_math_expression(self, visitor, "Multiply"),
            "/" => visit_as_math_expression(self, visitor, "Divide"),
            "%" => visit_as_math_expression(self, visitor, "Remainder"),
            "+" => visit_as_math_expression(self, visitor, "Add"),
            "-" => visit_as_math_expression(self, visitor, "Subtract"),
            "<" => visit_as_comparison_expression(self, visitor, ["Less-Than"]),
            "<=" => visit_as_comparison_expression(self, visitor, ["Less-Than", "Equal"]),
            ">" => visit_as_comparison_expression(self, visitor, ["Greater-Than"]),
            ">=" => visit_as_comparison_expression(self, visitor, ["Greater-Than", "Equal"]),
            "=" => {
                // TODO: a = b <=> Equal a b
                todo!()
            }
            "/=" => {
                // TODO: a = b <=> not (Equal a b)
                todo!()
            }
            "and" => {
                // TODO: a = b <=> And a {b}
                todo!()
            }
            "or" => {
                // TODO: a = b <=> Or a {b}
                todo!()
            }
            operator => panic!("invalid binary operator {operator:?}"),
        }
    }
}

fn visit_as_math_expression(
    expression: &BinaryExpression,
    visitor: &mut Visitor<'_>,
    trait_name: &str,
) -> NodeId {
    todo!()
}

fn visit_as_comparison_expression<const N: usize>(
    expression: &BinaryExpression,
    visitor: &mut Visitor<'_>,
    variant_names: [&str; N],
) -> NodeId {
    todo!()
}
