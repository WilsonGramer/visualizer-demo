use crate::{
    definitions::Definition,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{AssignmentStatement, Pattern, Range};
use wipple_compiler_trace::{Fact, NodeId};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for AssignmentStatement {
    fn name(&self) -> &'static str {
        "_assignment"
    }

    fn range(&self) -> Range {
        self.pattern.range()
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let value = visitor.child(&self.value, id, "assignmentValue");

        // Try assigning to an existing constant if possible
        if let Pattern::Variable(pattern) = &self.pattern {
            if let Some(constraint) =
                visitor.peek_name(&pattern.variable.value, |definition| match definition {
                    Definition::Constant(definition) => {
                        let ty = match definition.value {
                            Ok(_) => todo!(),
                            Err(ty) => ty,
                        };

                        definition.value = Err(value);

                        // Ensure the value is assignable to the constant's
                        // type
                        Some(Constraint::Ty(value, Ty::Of(ty)))
                    }
                    _ => None,
                })
            {
                visitor.constraint(constraint);
                visitor.fact(id, Fact::marker("assignmentToConstant"));
                return;
            }
        }

        let pattern = visitor.child(&self.pattern, value, "assignmentPattern");

        visitor.constraint(Constraint::Ty(value, Ty::Of(pattern)));
        visitor.fact(id, Fact::marker("assignmentToPattern"));
    }
}
