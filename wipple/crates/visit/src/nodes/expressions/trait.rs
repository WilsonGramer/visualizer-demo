use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use visualizer::{Constraint, Instantiation, Substitutions};
use wipple_db::NodeId;
use wipple_syntax::{Range, TraitExpression};

impl Visit for TraitExpression {
    fn name(&self) -> &'static str {
        "trait"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let definition =
            visitor.resolve_name(&self.r#type.value, id, |definition| match definition {
                Definition::Type(_) => todo!(),
                Definition::Trait(definition) => Some((definition.node, "resolvedTraitName")),
                _ => None,
            });

        if let Some(definition) = definition {
            visitor.constraint(Constraint::Instantiation(Instantiation {
                source: id,
                node: id,
                definition,
                substitutions: Substitutions::replace_all(),
            }));
        } else {
            visitor.fact(id, "unresolvedTraitName", ());
        }
    }
}
