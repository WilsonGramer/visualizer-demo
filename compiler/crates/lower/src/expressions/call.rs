use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{CallExpression, Expression};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{CallNode, DefinitionNode};

/// A function call.
pub const FUNCTION_CALL: Rule = Rule::new("function call");

/// The function in a function call.
pub const FUNCTION_IN_FUNCTION_CALL: Rule = Rule::new("function in function call");

/// An input in a function call.
pub const INPUT_IN_FUNCTION_CALL: Rule = Rule::new("input in function call");

/// A number with a unit.
pub const UNIT_CALL: Rule = Rule::new("unit call");

/// The number component.
pub const NUMBER_IN_UNIT_CALL: Rule = Rule::new("number in unit call");

/// The unit component.
pub const UNIT_IN_UNIT_CALL: Rule = Rule::new("unit in unit call");

impl Visit for CallExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            let unit = match self.inputs.as_slice() {
                [Expression::VariableName(input)] => {
                    Some((&input.range, input.variable.source.as_str()))
                }
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

                        let function = visitor.typed_node(
                            Some((id, UNIT_IN_UNIT_CALL)),
                            unit_range,
                            |_visitor, _id| {
                                (
                                    DefinitionNode {
                                        definition,
                                        constraints,
                                    },
                                    UNIT_IN_UNIT_CALL,
                                )
                            },
                        );

                        let input = self
                            .function
                            .visit(visitor, Some((id, NUMBER_IN_UNIT_CALL)));

                        return (
                            CallNode {
                                function,
                                inputs: vec![input],
                            },
                            UNIT_CALL,
                        );
                    }
                }
            }

            (
                CallNode {
                    function: self
                        .function
                        .visit(visitor, Some((id, FUNCTION_IN_FUNCTION_CALL))),
                    inputs: self
                        .inputs
                        .iter()
                        .map(|input| input.visit(visitor, Some((id, INPUT_IN_FUNCTION_CALL))))
                        .collect::<Vec<_>>(),
                },
                FUNCTION_CALL,
            )
        })
    }
}
