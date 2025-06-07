use crate::{Visit, Visitor};
use wipple_compiler_syntax::PlaceholderType;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

rule! {
    /// A placeholder type.
    placeholder_type;
}

impl Visit for PlaceholderType {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |_visitor, _id| {
            (PlaceholderNode, rule::placeholder_type)
        })
    }
}
