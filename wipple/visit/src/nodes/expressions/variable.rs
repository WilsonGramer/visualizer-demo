use crate::{
    constraints::instantiate_constraints,
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use visualizer::db::NodeId;
use visualizer::typecheck::{Constraint, Instantiation, Substitutions, Ty};
use wipple_syntax::{Range, VariableExpression};

impl Visit for VariableExpression {
    fn name(&self) -> &'static str {
        "variable"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let constraint =
            visitor.resolve_name(&self.variable.value, id, |definition| match definition {
                Definition::Variable(definition) => Some((
                    Constraint::Ty(id, Ty::Of(definition.node)),
                    "resolvedVariableName",
                )),
                Definition::Constant(definition) => Some((
                    Constraint::Instantiation(Instantiation {
                        substitutions: Substitutions::replace_all(),
                        constraints: instantiate_constraints(
                            id,
                            definition.constraints.iter().cloned(),
                        )
                        .collect(),
                    }),
                    "resolvedConstantName",
                )),
                _ => None,
            });

        if let Some(constraint) = constraint {
            visitor.constraint(constraint);
        } else {
            visitor.fact(id, "unresolvedVariableName", ());
        }
    }
}
