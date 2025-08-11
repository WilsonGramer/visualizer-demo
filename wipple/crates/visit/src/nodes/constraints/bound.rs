use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::{Bound, Constraint, Instantiation, Substitutions};
use wipple_db::NodeId;
use wipple_syntax::{BoundConstraint, Range};

impl Visit for BoundConstraint {
    fn name(&self) -> &'static str {
        "boundConstraint"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let Some((trait_node, trait_parameters)) =
            visitor.resolve_name(&self.r#trait.value, id, |definition| match definition {
                Definition::Trait(definition) => Some((
                    (definition.node, definition.parameters.clone()),
                    "resolvedTraitInBound",
                )),
                _ => None,
            })
        else {
            visitor.fact(id, "unresolvedTraitInBound", ());
            return;
        };

        let parameters = self
            .parameters
            .iter()
            .map(|ty| visitor.child(ty, id, "parameterInBound"));

        // TODO: Ensure `parameters` has the right length
        let substitutions = trait_parameters
            .into_iter()
            .zip(parameters)
            .collect::<BTreeMap<_, _>>();

        visitor.current_definition().lazy_constraint(move |node| {
            Constraint::Bound(Bound(Instantiation {
                source: node,
                node: id,
                definition: trait_node,
                substitutions: Substitutions::from(substitutions.clone()),
            }))
        });
    }
}
