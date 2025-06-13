use crate::{Visit, Visitor};
use wipple_compiler_syntax::WildcardPattern;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::ConstraintNode;

rule! {
    /// A wildcard pattern.
    wildcard_pattern: Typed;
}

impl Visit for WildcardPattern {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, _id| {
            (
                ConstraintNode {
                    value: visitor.parent(),
                    constraints: Vec::new(),
                },
                rule::wildcard_pattern,
            )
        })
    }
}
