use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NameExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{DefinitionNode, Node, PlaceholderNode};


    /// A name expression.
pub const NAME: Rule = Rule::new("name");

    /// A name that resolved to a value.
pub const RESOLVED_NAME: Rule = Rule::new("resolved_name");

    /// A name that was not resolved to a value.
pub const UNRESOLVED_NAME: Rule = Rule::new("unresolved_name");


impl Visit for NameExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            if let Some((definition, constraints)) =
                visitor.resolve_name(&self.variable.source, id, NAME, |definition| {
                    match definition {
                        Definition::Variable { node, .. } => Some((*node, Vec::new())),
                        Definition::Constant {
                            node, constraints, ..
                        } => Some((*node, constraints.clone())),
                        _ => None,
                    }
                })
            {
                (
                    DefinitionNode {
                        definition,
                        constraints,
                    }
                    .boxed(),
                    RESOLVED_NAME,
                )
            } else {
                (PlaceholderNode.boxed(), UNRESOLVED_NAME)
            }
        })
    }
}
