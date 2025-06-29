use crate::{Visit, Visitor};
use wipple_compiler_syntax::WildcardPattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

pub const WILDCARD_PATTERN: Rule = Rule::new("wildcard pattern");

impl Visit for WildcardPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.range, |_visitor, _id| {
            (PlaceholderNode, WILDCARD_PATTERN)
        })
    }
}
