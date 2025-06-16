use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NameExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::{DefinitionNode, Node, PlaceholderNode};

rule! {
    /// A name expression.
    name: Extra;

    /// A name that resolved to a value.
    resolved_name: Extra;

    /// A name that was not resolved to a value.
    unresolved_name: Extra;
}

impl Visit for NameExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            if let Some((definition, constraints)) =
                visitor.resolve_name(&self.variable.source, id, rule::name, |definition| {
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
                    rule::resolved_name.erased(),
                )
            } else {
                (PlaceholderNode.boxed(), rule::unresolved_name.erased())
            }
        })
    }
}
