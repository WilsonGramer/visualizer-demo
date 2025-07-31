use crate::{
    attributes::{AttributeParser, ConstantAttributes},
    definitions::{ConstantDefinition, Definition},
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{ConstantDefinitionStatement, Constraints, Range};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for ConstantDefinitionStatement {
    fn rule(&self) -> Rule {
        "constant definition".into()
    }

    fn range(&self) -> Range {
        self.name.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let attributes =
            ConstantAttributes::parse(visitor, &mut AttributeParser::new(id, &self.attributes));

        let (ty, constraints) = visitor.with_definition(|visitor| {
            visitor.current_definition().implicit_type_parameters = true;

            let ty = visitor.child(&self.constraints.r#type, id, "type in constant definition");

            visitor
                .current_definition()
                .lazy_constraint(move |node| Constraint::Ty(node, Ty::Of(ty)));

            if let Some(Constraints(constraints)) = &self.constraints.constraints {
                for constraint in constraints {
                    visitor.child(constraint, id, "constraint in constant definition");
                }
            }

            (ty, visitor.current_definition().take_constraints())
        });

        visitor.define_name(
            &self.name.value,
            Definition::Constant(ConstantDefinition {
                node: id,
                comments: self.comments.clone(),
                attributes,
                constraints,
                value: Err(ty),
            }),
        );

        visitor.constraint(Constraint::Ty(id, Ty::Of(ty)));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
