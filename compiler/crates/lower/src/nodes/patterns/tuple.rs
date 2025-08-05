use crate::{
    constraints::constraints_for_tuple,
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{Range, TuplePattern};
use wipple_visualizer_typecheck::NodeId;

impl Visit for TuplePattern {
    fn name(&self) -> &'static str {
        "tuplePattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let elements = self
            .elements
            .iter()
            .map(|element| visitor.child(element, id, "tuplePatternElement"))
            .collect::<Vec<_>>();

        visitor.constraint(constraints_for_tuple(id, elements));
    }
}
