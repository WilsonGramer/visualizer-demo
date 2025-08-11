use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::{Constraint, Ty};
use wipple_db::NodeId;
use wipple_syntax::{NumberExpression, Range};

impl Visit for NumberExpression {
    fn name(&self) -> &'static str {
        "number"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let number_ty = visitor.resolve_name("Number", id, |definition| match definition {
            Definition::Type(definition) => Some((definition.node, "number")),
            _ => None,
        });

        if let Some(number_ty) = number_ty {
            visitor.constraint(Constraint::Ty(
                id,
                Ty::Named {
                    name: number_ty,
                    parameters: BTreeMap::new(),
                },
            ));
        } else {
            visitor.fact(id, "missingNumberType", ());
        }
    }
}
