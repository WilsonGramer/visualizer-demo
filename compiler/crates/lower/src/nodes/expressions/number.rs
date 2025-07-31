use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{NumberExpression, Range};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for NumberExpression {
    fn rule(&self) -> Rule {
        "number".into()
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
            visitor.rule(id, "missing number type");
        }
    }

    fn is_typed(&self) -> bool {
        true
    }
}
