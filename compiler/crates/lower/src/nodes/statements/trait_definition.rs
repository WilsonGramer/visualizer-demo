use crate::{
    attributes::{AttributeParser, TraitAttributes},
    definitions::{Definition, TraitDefinition, TypeParameterDefinition},
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Constraints, Range, TraitDefinitionStatement};
use wipple_compiler_trace::NodeId;
use wipple_compiler_typecheck::constraints::{Bound, Constraint, Substitutions, Ty};

impl Visit for TraitDefinitionStatement {
    fn name(&self) -> &'static str {
        "traitDefinition"
    }

    fn range(&self) -> Range {
        self.name.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let attributes =
            TraitAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

        let parameters = self
            .parameters
            .as_ref()
            .map(|parameters| parameters.0.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|parameter| {
                let id = visitor.child(
                    &(parameter.range, "parameterName"),
                    id,
                    "parameterInTraitDefinition",
                );

                visitor.define_name(
                    &parameter.value,
                    Definition::TypeParameter(TypeParameterDefinition { node: id }),
                );

                visitor.constraint(Constraint::Ty(id, Ty::Parameter(id)));

                id
            })
            .collect::<Vec<_>>();

        let (ty, constraints) = visitor.with_definition(|visitor| {
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

            (ty, visitor.current_definition().take_constraints())
        });

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
    }

    fn is_typed(&self) -> bool {
        true
    }
}
