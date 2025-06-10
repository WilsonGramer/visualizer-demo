use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::{FunctionNode, PlaceholderNode};

rule! {
    /// A function expression.
    function: Typed;

    /// An input to a function expression.
    function_input: Typed;

    /// The output of a function expression.
    function_output: Typed;
}

impl Visit for FunctionExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            visitor.push_scope();

            let inputs = self
                .inputs
                .iter()
                .map(|input| {
                    let target = visitor.node(
                        Some((id, rule::function_input)),
                        input.range(),
                        |_visitor, _id| (PlaceholderNode, rule::function_input),
                    );

                    input.visit(visitor, Some((target, rule::function_input)))
                })
                .collect::<Vec<_>>();

            let output = self
                .output
                .visit(visitor, Some((id, rule::function_output)));

            visitor.pop_scope();

            (FunctionNode { inputs, output }, rule::function)
        })
    }
}
