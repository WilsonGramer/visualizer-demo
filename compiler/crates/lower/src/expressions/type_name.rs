use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::TypeNameExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{ConstraintNode, Node, PlaceholderNode};

pub const TYPE_NAME: Rule = Rule::new("type name");

pub const RESOLVED_TYPE_NAME: Rule = Rule::new("resolved type name");

pub const RESOLVED_TRAIT_NAME: Rule = Rule::new("resolved trait name");

pub const UNRESOLVED_TYPE_NAME: Rule = Rule::new("unresolved type name");

impl Visit for TypeNameExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            visitor.push_scope(id);

            let constraints =
                visitor.resolve_name(&self.r#type.source, id, |definition| match definition {
                    Definition::Type(_) => todo!(),
                    Definition::Trait(definition) => {
                        Some((definition.constraints.clone(), RESOLVED_TRAIT_NAME))
                    }
                    _ => None,
                });

            visitor.pop_scope();

            if let Some((constraints, rule)) = constraints {
                (
                    ConstraintNode {
                        value: id,
                        constraints,
                    }
                    .boxed(),
                    rule,
                )
            } else {
                (PlaceholderNode.boxed(), UNRESOLVED_TYPE_NAME)
            }
        })
    }
}
