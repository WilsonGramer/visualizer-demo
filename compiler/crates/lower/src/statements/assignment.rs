use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{AssignmentStatement, Pattern};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static ASSIGNMENT_VALUE: Rule = Rule::new("assignment value");
pub static ASSIGNMENT_PATTERN: Rule = Rule::new("assignment pattern");
pub static ASSIGNMENT_TO_CONSTANT: Rule = Rule::new("assignment to constant");
pub static ASSIGNMENT_TO_PATTERN: Rule = Rule::new("assignment to pattern");

impl Visit for AssignmentStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let value = self.value.visit(visitor, (id, ASSIGNMENT_VALUE));

            // Try assigning to an existing constant if possible
            if let Pattern::Variable(pattern) = &self.pattern {
                if let Some(node) =
                    visitor.peek_name(&pattern.variable.value, |definition| match definition {
                        Definition::Constant(definition) => {
                            if definition.value.is_ok() {
                                todo!();
                            }

                            let ty = match definition.value {
                                Ok(_) => todo!(),
                                Err(ty) => ty,
                            };

                            definition.value = Err(value);

                            // Ensure the value is assignable to the constant's
                            // type
                            Some(AnnotateNode {
                                value,
                                annotations: vec![Annotation::Node(ty)],
                            })
                        }
                        _ => None,
                    })
                {
                    return (node, ASSIGNMENT_TO_CONSTANT);
                }
            }

            let pattern = self.pattern.visit(visitor, (value, ASSIGNMENT_PATTERN));

            (
                AnnotateNode {
                    value,
                    annotations: vec![Annotation::Node(pattern)],
                },
                ASSIGNMENT_TO_PATTERN,
            )
        })
    }
}
