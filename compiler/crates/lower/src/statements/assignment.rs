use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::{AssignmentStatement, Pattern};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static ASSIGNMENT_VALUE: Rule = Rule::new("assignment_value");
pub static ASSIGNMENT_PATTERN: Rule = Rule::new("assignment_pattern");
pub static ASSIGNMENT: Rule = Rule::new("assignment");

impl Visit for AssignmentStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let value = self.value.visit(visitor, (id, ASSIGNMENT_VALUE));

            // Try assigning to an existing constant if possible
            if let Pattern::Variable(pattern) = &self.pattern {
                if let Some(node) =
                    visitor.peek_name(&pattern.variable.value, |definition| match definition {
                        Definition::Constant(definition) => {
                            if definition.assigned {
                                todo!();
                            }

                            definition.assigned = true;

                            // Ensure the value is assignable to the constant's type
                            Some(AnnotateNode {
                                value: definition.ty,
                                definition: Annotation::Node(value),
                            })
                        }
                        _ => None,
                    })
                {
                    return (node, ASSIGNMENT);
                }
            }

            let pattern = self.pattern.visit(visitor, (value, ASSIGNMENT_PATTERN));

            (
                AnnotateNode {
                    value,
                    definition: Annotation::Node(pattern),
                },
                ASSIGNMENT,
            )
        })
    }
}
