use crate::{Visit, Visitor};
use wipple_compiler_syntax::PlaceholderExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

/// A placeholder expression.
pub const PLACEHOLDER: Rule = Rule::new("placeholder");

impl Visit for PlaceholderExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |_visitor, _id| {
            (PlaceholderNode, PLACEHOLDER)
        })
    }
}
