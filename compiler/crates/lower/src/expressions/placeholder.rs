use crate::{Visit, Visitor};
use wipple_compiler_syntax::PlaceholderExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::EmptyNode;

pub static PLACEHOLDER: Rule = Rule::new("placeholder");

impl Visit for PlaceholderExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |_visitor, _id| {
            (EmptyNode, PLACEHOLDER)
        })
    }
}
