use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{BoundConstraint, Range};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Bound, Constraint, Substitutions};

impl Visit for BoundConstraint {
    fn rule(&self) -> Rule {
        "bound constraint".into()
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
            visitor.rule(id, "unresolved trait in bound");
            return;
        };

        let parameters = self
            .parameters
            .iter()
            .map(|ty| visitor.child(ty, id, "parameter in bound"));

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
