use crate::{
    attributes::{AttributeParser, InstanceAttributes},
    definitions::{Definition, InstanceDefinition},
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::{Constraint, Instantiation, Substitutions, Ty};
use wipple_db::NodeId;
use wipple_syntax::{Constraints, InstanceDefinitionStatement, Range};

impl Visit for InstanceDefinitionStatement {
    fn name(&self) -> &'static str {
        "instanceDefinition"
    }

    fn range(&self) -> Range {
        self.constraints.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.with_definition(|visitor| {
            let attributes =
                InstanceAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

            visitor.push_scope(id);

            let Some((trait_node, trait_parameters)) =
                visitor.resolve_name(&self.constraints.bound.r#trait.value, id, |definition| {
                    match definition {
                        Definition::Trait(definition) => Some((
                            (definition.node, definition.parameters.clone()),
                            "traitInInstanceDefinition",
                        )),
                        _ => None,
                    }
                })
            else {
                visitor.fact(id, "unresolvedTraitName", ());
                return;
            };

            visitor.fact(trait_node, "instance", id);

            visitor.current_definition().implicit_type_parameters = true;

            let parameters = self
                .constraints
                .bound
                .parameters
                .iter()
                .map(|ty| visitor.child(ty, id, "parameterInInstanceDefinition"));

            // TODO: Ensure `parameters` has the right length
            let substitutions = Substitutions::from(
                trait_parameters
                    .into_iter()
                    .zip(parameters)
                    .collect::<BTreeMap<_, _>>(),
            );

            visitor
                .current_definition()
                .lazy_constraint(move |node| Constraint::Ty(node, Ty::Of(id)));

            if let Some(Constraints(constraints)) = &self.constraints.constraints {
                for constraint in constraints {
                    visitor.child(constraint, id, "constraintInInstanceDefinition");
                }
            }

            visitor.current_definition().implicit_type_parameters = false;
            visitor.current_definition().is_typed = true;

            let value = visitor.child(&self.value, id, "valueInInstanceDefinition");

            visitor.pop_scope();

            let constraints = visitor.current_definition().take_constraints();

            visitor.define_instance(InstanceDefinition {
                node: id,
                comments: self.comments.clone(),
                attributes,
                tr: trait_node,
                value,
            });

            visitor.fact(id, "constraints", constraints);
            visitor.fact(id, "substitutions", substitutions.clone());

            visitor.constraints(vec![
                Constraint::Instantiation(Instantiation {
                    source: id,
                    substitutions,
                    constraints: vec![Constraint::Ty(id, Ty::Of(trait_node))],
                }),
                Constraint::Ty(id, Ty::Of(value)),
            ]);
        });
    }
}
