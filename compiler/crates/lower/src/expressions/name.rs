use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NameExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{DefinitionNode, Node, PlaceholderNode},
};

rule! {
    /// A name expression.
    name;

    /// A name that resolved to a value.
    resolved_name;

    /// A name that was not resolved to a value.
    unresolved_name;
}

impl Visit for NameExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let definition = match visitor.resolve_name(&self.variable.source, id, rule::name) {
                Some(Definition::Variable { node, .. }) => Some((*node, Vec::new())),
                Some(Definition::Constant {
                    node, constraints, ..
                }) => Some((*node, constraints.clone())),
                // TODO: Create constructor or use directly?
                // Some(Definition::Trait {
                //     node, constraints, ..
                // }) => Some((*node, constraints.clone())),
                _ => None,
            };

            if let Some((definition, constraints)) = definition {
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
