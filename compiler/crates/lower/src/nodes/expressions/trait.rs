use crate::{
    constraints::instantiate_constraints,
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{Range, TraitExpression};
use wipple_visualizer_typecheck::{Constraint, Fact, Instantiation, NodeId, Substitutions};

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
                substitutions: Substitutions::replace_all(),
                constraints: instantiate_constraints(id, constraints).collect(),
            }));
        } else {
            visitor.fact(id, Fact::new("unresolvedTraitName", ()));
        }
    }

}
