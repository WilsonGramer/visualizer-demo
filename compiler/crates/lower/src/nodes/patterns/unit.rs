use crate::{
    constraints::constraints_for_unit,
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{Range, UnitPattern};
use wipple_visualizer_typecheck::NodeId;

impl Visit for UnitPattern {
    fn name(&self) -> &'static str {
        "unitPattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.constraints(constraints_for_unit(id));
    }
}
