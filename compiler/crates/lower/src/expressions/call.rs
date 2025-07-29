use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{CallExpression, Expression};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::Substitutions,
    nodes::{AnnotateNode, Annotation, CallNode},
};

pub static FUNCTION_CALL: Rule = Rule::new("function call");
pub static FUNCTION_IN_FUNCTION_CALL: Rule = Rule::new("function in function call");
pub static INPUT_IN_FUNCTION_CALL: Rule = Rule::new("input in function call");
pub static UNIT_CALL: Rule = Rule::new("unit call");
pub static NUMBER_IN_UNIT_CALL: Rule = Rule::new("number in unit call");
pub static UNIT_IN_UNIT_CALL: Rule = Rule::new("unit in unit call");

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
                if let Some((attributes, annotations)) =
                    visitor.peek_name(unit_name, |definition| match definition {
                        Definition::Constant(definition) => {
                            Some((&definition.attributes, definition.annotations.clone()))
                        }
                        _ => None,
                    })
                {
                    if attributes.unit {
                        let function = visitor.typed_node(
                            (id, UNIT_IN_UNIT_CALL),
                            unit_range,
                            |_visitor, id| {
                                (
                                    AnnotateNode {
                                        value: id,
                                        annotations: vec![Annotation::Instantiate {
                                            annotations,
                                            substitutions: Substitutions::replace_all(),
                                        }],
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
