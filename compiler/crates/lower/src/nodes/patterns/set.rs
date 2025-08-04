use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, SetPattern};
use wipple_compiler_trace::{Fact, NodeId};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for SetPattern {
    fn name(&self) -> &'static str {
        "setPattern"
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
                _ => None,
            });

        if let Some(constraint) = constraint {
            visitor.constraint(constraint);
        } else {
            visitor.fact(id, Fact::marker("unresolvedVariableName"));
        }
    }
}
