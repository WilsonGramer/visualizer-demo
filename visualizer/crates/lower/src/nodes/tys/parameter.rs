use crate::{
    definitions::{Definition, TypeParameterDefinition},
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{ParameterType, Range};
use wipple_visualizer_typecheck::{
    Constraint, Ty,
    Fact, NodeId,
};

impl Visit for ParameterType {
    fn name(&self) -> &'static str {
        "parameterType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let existing = visitor.resolve_name(&self.name.value, id, |definition| match definition {
            Definition::TypeParameter(definition) => Some((definition.node, "parameterType")),
            _ => None,
        });

        match existing {
            Some(node) => {
                visitor.constraint(Constraint::Ty(id, Ty::Of(node)));
            }
            None => {
                if visitor
                    .try_current_definition()
                    .is_some_and(|definition| definition.implicit_type_parameters)
                {
                    visitor.define_name(
                        &self.name.value,
                        Definition::TypeParameter(TypeParameterDefinition { node: id }),
                    );

                    visitor.constraint(Constraint::Ty(id, Ty::Parameter(id)));
                } else {
                    visitor.fact(id, Fact::new("unresolvedParameterType", ()));
                }
            }
        }
    }
}
