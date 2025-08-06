use crate::{
    constraints::{constraints_for_call, instantiate_constraints},
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use wipple_db::NodeId;
use visualizer::{Constraint, Instantiation, Substitutions};
use wipple_syntax::{CallExpression, Expression, Range};

impl Visit for CallExpression {
    fn name(&self) -> &'static str {
        "functionCall"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let unit = match self.inputs.as_slice() {
            [Expression::Variable(input)] => Some((input.range, input.variable.value.as_str())),
            _ => None,
        };

        // If `inputs` has a single element with the `[unit]` attribute,
        // flip the order
        if let Some((unit_range, unit_name)) = unit {
            if let Some((attributes, constraints)) =
                visitor.peek_name(unit_name, |definition| match definition {
                    Definition::Constant(definition) => {
                        Some((&definition.attributes, definition.constraints.clone()))
                    }
                    _ => None,
                })
            {
                if attributes.unit {
                    let function = visitor.child(&(unit_range, "unitName"), id, "unitInUnitCall");

                    visitor.constraint(Constraint::Instantiation(Instantiation {
                        substitutions: Substitutions::replace_all(),
                        constraints: instantiate_constraints(id, constraints).collect(),
                    }));

                    let input = visitor.child(self.function.as_ref(), id, "numberInUnitCall");

                    visitor.constraints(constraints_for_call(function, [input], id));

                    return;
                }
            }
        }

        let function = visitor.child(self.function.as_ref(), id, "functionInFunctionCall");

        let inputs = self
            .inputs
            .iter()
            .map(|input| visitor.child(input, id, "inputInFunctionCall"))
            .collect::<Vec<_>>();

        visitor.constraints(constraints_for_call(function, inputs, id));
    }
}
