use crate::{Visit, Visitor};
use wipple_compiler_syntax::WildcardPattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::EmptyNode;

pub static WILDCARD_PATTERN: Rule = Rule::new("wildcard pattern");

impl Visit for WildcardPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |_visitor, _id| {
            (EmptyNode, WILDCARD_PATTERN)
        })
    }
}
