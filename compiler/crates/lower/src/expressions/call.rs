use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{CallExpression, Expression};
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::{CallNode, DefinitionNode};

rule! {
    /// A function call.
    function_call: Typed;

    /// The function in a function call.
    function_in_function_call: Typed;

    /// An input in a function call.
    input_in_function_call: Typed;

    /// A number with a unit.
    unit_call: Typed;

    /// The number component.
    number_in_unit_call: Typed;

    /// The unit component.
    unit_in_unit_call: Typed;
}

impl Visit for CallExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let unit = match self.inputs.as_slice() {
                [Expression::Name(input)] => Some((&input.range, input.variable.source.as_str())),
                _ => None,
            };

            // If `inputs` has a single element with the `[unit]` attribute,
            // flip the order
            if let Some((unit_range, unit_name)) = unit {
                if let Some((definition, attributes, constraints)) =
                    visitor.peek_name(unit_name, |definition| match definition {
                        Definition::Constant {
                            node: definition,
                            attributes,
                            constraints,
                            ..
                        } => Some((definition, attributes, constraints)),
                        _ => None,
                    })
                {
                    if attributes.unit {
                        let definition = *definition;
                        let constraints = constraints.clone();

                        let function = visitor.node(
                            Some((id, rule::unit_in_unit_call)),
                            unit_range,
                            |_visitor, _id| {
                                (
                                    DefinitionNode {
                                        definition,
                                        constraints,
                                    },
                                    rule::unit_in_unit_call,
                                )
                            },
                        );

                        let input = self
                            .function
                            .visit(visitor, Some((id, rule::number_in_unit_call)));

                        return (
                            CallNode {
                                function,
                                inputs: vec![input],
                            },
                            rule::unit_call.erased(),
                        );
                    }
                }
            }

            (
                CallNode {
                    function: self
                        .function
                        .visit(visitor, Some((id, rule::function_in_function_call))),
                    inputs: self
                        .inputs
                        .iter()
                        .map(|input| input.visit(visitor, Some((id, rule::input_in_function_call))))
                        .collect::<Vec<_>>(),
                },
                rule::function_call.erased(),
            )
        })
    }
}
