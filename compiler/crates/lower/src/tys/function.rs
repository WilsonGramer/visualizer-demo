use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, PlaceholderNode},
};

pub const FUNCTION_TYPE: Rule = Rule::new("function type");

pub const FUNCTION_TYPE_INPUT: Rule = Rule::new("function type input");

pub const FUNCTION_TYPE_OUTPUT: Rule = Rule::new("function type output");

impl Visit for FunctionType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let inputs = self
                .inputs
                .0
                .iter()
                .map(|input| {
                    let node = visitor.node(
                        (id, FUNCTION_TYPE_INPUT),
                        input.range(),
                        |visitor, target| {
                            visitor.with_target(target, |visitor| {
                                Ty::Of(input.visit(visitor, (id, FUNCTION_TYPE_INPUT)))
                            });

                            (PlaceholderNode, FUNCTION_TYPE_INPUT)
                        },
                    );

                    Ty::Of(node)
                })
                .collect::<Vec<_>>();

            let output = visitor.node(
                (id, FUNCTION_TYPE_OUTPUT),
                self.output.range(),
                |visitor, target| {
                    visitor.with_target(target, |visitor| {
                        Ty::Of(self.output.visit(visitor, (id, FUNCTION_TYPE_OUTPUT)))
                    });

                    (PlaceholderNode, FUNCTION_TYPE_OUTPUT)
                },
            );

            (
                ConstraintNode {
                    value: visitor.target(),
                    constraints: vec![Constraint::Ty(Ty::Function {
                        inputs,
                        output: Box::new(Ty::Of(output)),
                    })],
                },
                FUNCTION_TYPE,
            )
        })
    }
}
