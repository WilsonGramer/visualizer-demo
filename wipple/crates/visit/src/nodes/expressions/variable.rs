use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use visualizer::{Constraint, Instantiation, Substitutions, Ty};
use wipple_db::NodeId;
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
                        source: id,
                        node: id,
                        definition: definition.node,
                        substitutions: Substitutions::replace_all(),
                    }),
                    "resolvedConstantName",
                )),
                _ => None,
            });

        if let Some(constraint) = constraint {
            visitor.constraint(constraint);
        }
    }
}
