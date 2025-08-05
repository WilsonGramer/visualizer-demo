use crate::{
    attributes::{AttributeParser, TraitAttributes},
    definitions::{Definition, TraitDefinition, TypeParameterDefinition},
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{Constraints, Range, TraitDefinitionStatement};
use wipple_visualizer_typecheck::{Bound, Constraint, NodeId, Substitutions, Ty};

impl Visit for TraitDefinitionStatement {
    fn name(&self) -> &'static str {
        "traitDefinition"
    }

    fn range(&self) -> Range {
        self.name.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.with_definition(|visitor| {
            let attributes =
                TraitAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

            let parameters = self
                .parameters
                .as_ref()
                .map(|parameters| parameters.0.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|parameter| {
                    let node = visitor.child(
                        &(parameter.range, "parameterName"),
                        id,
                        "parameterInTraitDefinition",
                    );

                    visitor.hide(node);

                    visitor.define_name(
                        &parameter.value,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    visitor.constraint(Constraint::Ty(node, Ty::Parameter(node)));

                    node
                })
                .collect::<Vec<_>>();

            let ty = visitor.child(&self.constraints.r#type, id, "typeInTraitDefinition");

            visitor
                .current_definition()
                .lazy_constraint(move |node| Constraint::Ty(node, Ty::Of(ty)));

            visitor.current_definition().lazy_constraint(move |node| {
                Constraint::Bound(Bound {
                    node,
                    tr: id,
                    substitutions: Substitutions::replace_all(),
                })
            });

            if let Some(Constraints(constraints)) = &self.constraints.constraints {
                for constraint in constraints {
                    visitor.child(constraint, id, "constraintInTraitDefinition");
                }
            }

            let constraints = visitor.current_definition().take_constraints();

            visitor.define_name(
                &self.name.value,
                Definition::Trait(TraitDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters,
                    constraints,
                }),
            );

            visitor.constraint(Constraint::Ty(id, Ty::Of(ty)));
        });
    }
}
