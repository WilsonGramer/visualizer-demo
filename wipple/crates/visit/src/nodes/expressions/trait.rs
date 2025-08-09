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
        let constraints =
            visitor.resolve_name(&self.r#type.value, id, |definition| match definition {
                Definition::Type(_) => todo!(),
                Definition::Trait(definition) => {
                    Some((definition.constraints.clone(), "resolvedTraitName"))
                }
                _ => None,
            });

        if let Some(constraints) = constraints {
            visitor.constraint(Constraint::Instantiation(Instantiation {
                source: id,
                substitutions: Substitutions::replace_all(),
                constraints: constraints.resolve_for(id),
            }));
        } else {
            visitor.fact(id, "unresolvedTraitName", ());
        }
    }
}
