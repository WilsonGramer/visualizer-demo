use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::db::NodeId;
use visualizer::typecheck::{Constraint, Ty};
use wipple_syntax::{NamedType, Range};

impl Visit for NamedType {
    fn name(&self) -> &'static str {
        "namedType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let Some(type_node) =
            visitor.resolve_name(&self.name.value, id, |definition| match definition {
                Definition::Type(definition) => Some((definition.node, "resolvedNamedType")),
                _ => None,
            })
        else {
            visitor.fact(id, "unresolvedNamedType", ());
            return;
        };

        // TODO: Ensure `definition.parameters` is empty

        visitor.constraint(Constraint::Ty(
            id,
            Ty::Named {
                name: type_node,
                parameters: BTreeMap::new(),
            },
        ));
    }
}
