use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::{Constraint, Ty};
use wipple_db::NodeId;
use wipple_syntax::{ParameterizedType, ParameterizedTypeElement, Range};

impl Visit for ParameterizedType {
    fn name(&self) -> &'static str {
        "parameterizedType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let Some((type_node, type_parameters)) =
            visitor.resolve_name(&self.name.value, id, |definition| match definition {
                Definition::Type(definition) => Some((
                    (definition.node, definition.parameters.clone()),
                    "resolvedParameterizedType",
                )),
                _ => None,
            })
        else {
            visitor.fact(id, "unresolvedParameterizedType", ());
            return;
        };

        let parameters = self
            .parameters
            .iter()
            .map(|ParameterizedTypeElement(ty)| {
                visitor.child(ty, id, "parameterInParameterizedType")
            })
            .collect::<Vec<_>>();

        // TODO: Ensure `parameters` has the right length

        visitor.constraint(Constraint::Ty(
            id,
            Ty::Named {
                name: type_node,
                parameters: type_parameters
                    .iter()
                    .cloned()
                    .zip(parameters.into_iter().map(Ty::Of))
                    .collect::<BTreeMap<_, _>>(),
            },
        ));
    }
}
