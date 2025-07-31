use crate::{
    constraints::constraints_for_unit,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, UnitType};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for UnitType {
    fn rule(&self) -> Rule {
        "unit type".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.constraints(constraints_for_unit(id));
    }
}
