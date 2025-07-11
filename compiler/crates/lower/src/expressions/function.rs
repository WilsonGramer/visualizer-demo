use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{FunctionNode, PlaceholderNode};

pub const FUNCTION: Rule = Rule::new("function");

pub const FUNCTION_INPUT: Rule = Rule::new("function input");

pub const FUNCTION_OUTPUT: Rule = Rule::new("function output");

impl Visit for FunctionExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            visitor.push_scope(id);

            let inputs = self
                .inputs
                .0
                .iter()
                .map(|input| {
                    let target = visitor.placeholder_node((id, FUNCTION_INPUT), input.range());

                    visitor.with_target(target, |visitor| {
                        input.visit(visitor, (target, FUNCTION_INPUT))
                    })
                })
                .collect::<Vec<_>>();

            let output = self.output.visit(visitor, (id, FUNCTION_OUTPUT));

            visitor.pop_scope();

            (FunctionNode { inputs, output }, FUNCTION)
        })
    }
}
