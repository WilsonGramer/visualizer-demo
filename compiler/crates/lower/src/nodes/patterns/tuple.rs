use crate::{
    constraints::constraints_for_tuple,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, TuplePattern};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for TuplePattern {
    fn rule(&self) -> Rule {
        "tuple pattern".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let elements = self
            .elements
            .iter()
            .map(|element| visitor.child(element, id, "tuple pattern element"))
            .collect::<Vec<_>>();

        visitor.constraint(constraints_for_tuple(id, elements));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
