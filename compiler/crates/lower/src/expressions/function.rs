use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{FunctionNode, PlaceholderNode};

pub const FUNCTION: Rule = Rule::new("function");

pub const FUNCTION_INPUT: Rule = Rule::new("function input");

pub const FUNCTION_OUTPUT: Rule = Rule::new("function output");

impl Visit for FunctionExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            visitor.push_scope(id);

            let inputs = self
                .inputs
                .iter()
                .map(|input| {
                    let target = visitor.node(
                        Some((id, FUNCTION_INPUT)),
                        input.range(),
                        |_visitor, _id| (PlaceholderNode, FUNCTION_INPUT),
                    );

                    input.visit(visitor, Some((target, FUNCTION_INPUT)))
                })
                .collect::<Vec<_>>();

            let output = self.output.visit(visitor, Some((id, FUNCTION_OUTPUT)));

            visitor.pop_scope();

            (FunctionNode { inputs, output }, FUNCTION)
        })
    }
}
