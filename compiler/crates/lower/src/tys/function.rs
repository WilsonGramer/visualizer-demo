use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

/// A function type.
pub const FUNCTION_TYPE: Rule = Rule::new("function_type", &[]);

/// An input to a function type.
pub const FUNCTION_TYPE_INPUT: Rule = Rule::new("function_type_input", &[]);

/// The output of a function type.
pub const FUNCTION_TYPE_OUTPUT: Rule = Rule::new("function_type_output", &[]);

impl Visit for FunctionType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let inputs = self
                .inputs
                .iter()
                .map(|input| {
                    let input_target =
                        visitor.root_placeholder_node(input.range(), FUNCTION_TYPE_INPUT);

                    visitor.with_target(input_target, |visitor| {
                        Ty::Of(input.visit(visitor, Some((id, FUNCTION_TYPE_INPUT))))
                    })
                })
                .collect::<Vec<_>>();

            let output_target =
                visitor.root_placeholder_node(self.output.range(), FUNCTION_TYPE_OUTPUT);

            let output = visitor.with_target(output_target, |visitor| {
                Ty::Of(self.output.visit(visitor, Some((id, FUNCTION_TYPE_OUTPUT))))
            });

            (
                ConstraintNode {
                    value: visitor.target(),
                    constraints: vec![Constraint::Ty(Ty::Function {
                        inputs,
                        output: Box::new(output),
                    })],
                },
                FUNCTION_TYPE,
            )
        })
    }
}
