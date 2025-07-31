use crate::{
    attributes::{AttributeParser, TypeAttributes},
    definitions::{Definition, TypeDefinition, TypeParameterDefinition},
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::{Range, TypeDefinitionStatement};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for TypeDefinitionStatement {
    fn rule(&self) -> Rule {
        "type definition".into()
    }

    fn range(&self) -> Range {
        self.name.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let attributes =
            TypeAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

        let parameters = self
            .parameters
            .as_ref()
            .map(|parameters| parameters.0.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|parameter| {
                let node = visitor.child(&(parameter.range, "parameter name".into()), id, "parameter in type definition");

                visitor.define_name(
                    &parameter.value,
                    Definition::TypeParameter(TypeParameterDefinition { node }),
                );

                node
            })
            .collect::<Vec<_>>();

        let constraints = visitor.with_definition(|visitor| {
            visitor.current_definition().lazy_constraint(move |node| {
                Constraint::Ty(
                    node,
                    Ty::Named {
                        name: id,
                        parameters: BTreeMap::new(), // FIXME
                    },
                )
            });

            // Types don't have additional constraints

            visitor.current_definition().take_constraints()
        });

        visitor.define_name(
            &self.name.value,
            Definition::Type(TypeDefinition {
                node: id,
                comments: self.comments.clone(),
                attributes,
                parameters,
                constraints,
            }),
        );
    }
}
