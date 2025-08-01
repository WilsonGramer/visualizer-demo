use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{ParameterizedType, ParameterizedTypeElement, Range};
use wipple_compiler_trace::{Fact, NodeId};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

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
                    "resolved parameterized type",
                )),
                _ => None,
            })
        else {
            visitor.fact(id, Fact::marker("unresolvedParameterizedType"));
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
