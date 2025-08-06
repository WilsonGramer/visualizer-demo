use crate::{
    constraints::constraints_for_unit,
    visitor::{Visit, Visitor},
};
use visualizer::db::NodeId;
use wipple_syntax::{Range, UnitType};

impl Visit for UnitType {
    fn name(&self) -> &'static str {
        "unitType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.constraints(constraints_for_unit(id));
    }
}
