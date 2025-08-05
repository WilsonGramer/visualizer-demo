use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{BoundConstraint, Range};
use wipple_compiler_typecheck::{
    constraints::{Bound, Constraint, Substitutions},
    util::{Fact, NodeId},
};

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
                    "resolved trait in bound",
                )),
                _ => None,
            })
        else {
            visitor.fact(id, Fact::marker("unresolvedTraitInBound"));
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

        visitor
            .current_definition()
            .constraint(Constraint::Bound(Bound {
                node: id,
                tr: trait_node,
                substitutions: Substitutions::from(substitutions),
            }));
    }
}
