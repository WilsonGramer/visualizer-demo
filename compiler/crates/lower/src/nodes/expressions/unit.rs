use crate::{
    constraints::constraints_for_unit,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, UnitExpression};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for UnitExpression {
    fn rule(&self) -> Rule {
        "unit".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.constraints(constraints_for_unit(id));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
