use crate::{
    constraints::instantiate_constraints,
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, TraitExpression};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Instantiation, Substitutions};

impl Visit for TraitExpression {
    fn rule(&self) -> Rule {
        "trait".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let constraints =
            visitor.resolve_name(&self.r#type.value, id, |definition| match definition {
                Definition::Type(_) => todo!(),
                Definition::Trait(definition) => {
                    Some((definition.constraints.clone(), "resolved trait name"))
                }
                _ => None,
            });

        if let Some(constraints) = constraints {
            visitor.constraint(Constraint::Instantiation(Instantiation {
                substitutions: Substitutions::replace_all(),
                constraints: instantiate_constraints(id, constraints).collect(),
            }));
        } else {
            visitor.rule(id, "unresolved trait name");
        }
    }

    fn is_typed(&self) -> bool {
        true
    }
}
