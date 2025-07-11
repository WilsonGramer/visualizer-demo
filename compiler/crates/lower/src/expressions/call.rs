use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{CallExpression, Expression};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{CallNode, ConstraintNode},
};

pub const FUNCTION_CALL: Rule = Rule::new("function call");

pub const FUNCTION_IN_FUNCTION_CALL: Rule = Rule::new("function in function call");

pub const INPUT_IN_FUNCTION_CALL: Rule = Rule::new("input in function call");

pub const UNIT_CALL: Rule = Rule::new("unit call");

pub const NUMBER_IN_UNIT_CALL: Rule = Rule::new("number in unit call");

pub const UNIT_IN_UNIT_CALL: Rule = Rule::new("unit in unit call");

impl Visit for CallExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let unit = match self.inputs.as_slice() {
                [Expression::Variable(input)] => Some((input.range, input.variable.value.as_str())),
                _ => None,
            };

            // If `inputs` has a single element with the `[unit]` attribute,
            // flip the order
            if let Some((unit_range, unit_name)) = unit {
                if let Some((definition, attributes, constraints)) =
                    visitor.peek_name(unit_name, |definition| match definition {
                        Definition::Constant(definition) => Some((
                            definition.node,
                            &definition.attributes,
                            &definition.constraints,
                        )),
                        _ => None,
                    })
                {
                    if attributes.unit {
                        let constraints = constraints.clone();

                        let function = visitor.typed_node(
                            (id, UNIT_IN_UNIT_CALL),
                            unit_range,
                            |_visitor, id| {
                                (
                                    ConstraintNode {
                                        value: id,
                                        constraints: vec![Constraint::Ty(Ty::Of(definition))]
                                            .into_iter()
                                            .chain(constraints)
                                            .collect(),
                                    },
                                    UNIT_IN_UNIT_CALL,
                                )
                            },
                        );

                        let input = self.function.visit(visitor, (id, NUMBER_IN_UNIT_CALL));

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
                        .visit(visitor, (id, FUNCTION_IN_FUNCTION_CALL)),
                    inputs: self
                        .inputs
                        .iter()
                        .map(|input| input.visit(visitor, (id, INPUT_IN_FUNCTION_CALL)))
                        .collect::<Vec<_>>(),
                },
                FUNCTION_CALL,
            )
        })
    }
}
