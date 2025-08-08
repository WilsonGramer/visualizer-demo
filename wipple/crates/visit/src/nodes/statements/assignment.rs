use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use visualizer::{Constraint, Ty};
use wipple_db::NodeId;
use wipple_syntax::{AssignmentStatement, Pattern, Range};

impl Visit for AssignmentStatement {
    fn name(&self) -> &'static str {
        "assignment"
    }

    fn range(&self) -> Range {
        self.pattern.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let value = visitor.child(&self.value, id, "assignmentValue");

        // Try assigning to an existing constant if possible
        if let Pattern::Variable(pattern) = &self.pattern {
            if let Some((definition, constraint)) =
                visitor.peek_name(&pattern.variable.value, |definition| match definition {
                    Definition::Constant(definition) => {
                        let ty = match definition.value {
                            Ok(_) => todo!(),
                            Err(ty) => ty,
                        };

                        definition.value = Err(value);

                        // Ensure the value is assignable to the constant's
                        // type
                        Some((definition.node, Constraint::Ty(value, Ty::Of(ty))))
                    }
                    _ => None,
                })
            {
                visitor.fact(id, "assignmentToConstant", definition);
                visitor.constraint(constraint);
                return;
            }
        }

        let pattern = visitor.child(&self.pattern, id, "assignmentPattern");

        visitor.constraint(Constraint::Ty(value, Ty::Of(pattern)));
        visitor.fact(id, "assignmentToPattern", ());
    }

    fn hide(&self) -> bool {
        true
    }
}
