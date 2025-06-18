use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::TypeNameExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{ConstraintNode, Node, PlaceholderNode};

/// A type name expression.
pub const TYPE_NAME: Rule = Rule::new("type_name", &[]);

/// A type name that resolved to a value.
pub const RESOLVED_TYPE_NAME: Rule = Rule::new("resolved_type_name", &[]);

/// A type name that was not resolved to a value.
pub const UNRESOLVED_TYPE_NAME: Rule = Rule::new("unresolved_type_name", &[]);

impl Visit for TypeNameExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            visitor.push_scope(id);

            let constraints =
                visitor.resolve_name(&self.r#type.source, id, TYPE_NAME, |definition| {
                    match definition {
                        Definition::Type { .. } => todo!(),
                        Definition::Trait { constraints, .. } => Some(constraints.clone()),
                        _ => None,
                    }
                });

            visitor.pop_scope();

            if let Some(constraints) = constraints {
                (
                    ConstraintNode {
                        value: id,
                        constraints,
                    }
                    .boxed(),
                    RESOLVED_TYPE_NAME,
                )
            } else {
                (PlaceholderNode.boxed(), UNRESOLVED_TYPE_NAME)
            }
        })
    }
}
