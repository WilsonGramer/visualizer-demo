use crate::{Visit, Visitor};
use wipple_compiler_syntax::FunctionType;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

rule! {
    /// A function type.
    function_type: Typed;

    /// An input to a function type.
    function_type_input: Typed;

    /// The output of a function type.
    function_type_output: Typed;
}

impl Visit for FunctionType {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let inputs = self
                .inputs
                .iter()
                .map(|input| Ty::Of(input.visit(visitor, Some((id, rule::function_type_input)))))
                .collect::<Vec<_>>();

            let output = Ty::Of(
                self.output
                    .visit(visitor, Some((id, rule::function_type_output))),
            );

            (
                ConstraintNode {
                    value: id,
                    constraints: vec![Constraint::Ty(Ty::Function {
                        inputs,
                        output: Box::new(output),
                    })],
                },
                rule::function_type,
            )
        })
    }
}
