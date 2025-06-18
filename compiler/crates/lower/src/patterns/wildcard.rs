use crate::{Visit, Visitor};
use wipple_compiler_syntax::WildcardPattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::ConstraintNode;

/// A wildcard pattern.
pub const WILDCARD_PATTERN: Rule = Rule::new("wildcard_pattern", &[]);

impl Visit for WildcardPattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, _id| {
            (
                ConstraintNode {
                    value: visitor.parent(),
                    constraints: Vec::new(),
                },
                WILDCARD_PATTERN,
            )
        })
    }
}
