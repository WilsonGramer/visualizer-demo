use crate::{
    attributes::{AttributeParser, TypeAttributes},
    definitions::{Definition, TypeDefinition, TypeParameterDefinition},
    visitor::{Visit, Visitor},
};
use std::collections::BTreeMap;
use visualizer::{Constraint, Ty};
use wipple_db::NodeId;
use wipple_syntax::{Range, TypeDefinitionStatement};

impl Visit for TypeDefinitionStatement {
    fn name(&self) -> &'static str {
        "typeDefinition"
    }

    fn range(&self) -> Range {
        self.name.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.with_definition(|visitor| {
            let attributes =
                TypeAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

            visitor.push_scope(id);

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
                        "parameterInTypeDefinition",
                    );

                    visitor.hide(node);

                    visitor.define_name(
                        &parameter.value,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    node
                })
                .collect::<Vec<_>>();

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

            visitor.pop_scope();

            let constraints = visitor.current_definition().take_constraints();

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
        })
    }
}
