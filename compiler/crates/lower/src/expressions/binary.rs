use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::BinaryExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::{CallNode, DefinitionNode, Node, PlaceholderNode};

rule! {
    /// The operator in a binary operator expression.
    operator: Typed;

    /// An `=` operator expression.
    equal: Typed;

    /// The left side of an `=` operator.
    equal_operator_left: Typed;

    /// The right side of an `=` operator.
    equal_operator_right: Typed;

    /// The `Equal` trait isn't defined.
    missing_equal_trait: Typed;
}

impl Visit for BinaryExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
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
            "=" => visitor.node(parent, &self.range, |visitor, id| {
                let function = visitor.node(
                    Some((id, rule::operator)),
                    &self.operator.range,
                    |visitor, id| {
                        let equal_function =
                            visitor.resolve_name("Equal", id, rule::operator, |definition| {
                                match definition {
                                    Definition::Constant { node, .. } => Some(*node),
                                    _ => None,
                                }
                            });

                        match equal_function {
                            Some(equal_function) => (
                                DefinitionNode {
                                    definition: equal_function,
                                    constraints: Vec::new(),
                                }
                                .boxed(),
                                rule::operator.erased(),
                            ),
                            None => (PlaceholderNode.boxed(), rule::missing_equal_trait.erased()),
                        }
                    },
                );

                let inputs = [
                    (self.left.as_ref(), rule::equal_operator_left.erased()),
                    (self.right.as_ref(), rule::equal_operator_right.erased()),
                ]
                .into_iter()
                .map(|(input, rule)| input.visit(visitor, Some((id, rule))))
                .collect::<Vec<_>>();

                (CallNode { function, inputs }.boxed(), rule::equal.erased())
            }),
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
