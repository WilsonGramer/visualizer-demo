use crate::{Visit, Visitor};
use wipple_compiler_syntax::TuplePattern;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, PlaceholderNode, TupleElementNode},
};

rule! {
    /// A tuple pattern.
    tuple_pattern;

    /// The target of a tuple pattern.
    tuple_pattern_target;

    /// An element in a tuple pattern.
    tuple_pattern_element;
}

impl Visit for TuplePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            for (index, element) in self.elements.iter().enumerate() {
                let target = visitor.node(
                    Some((id, rule::tuple_pattern_target)),
                    &self.range,
                    |visitor, _id| {
                        (
                            TupleElementNode {
                                index,
                                count: self.elements.len(),
                                target: visitor.parent(),
                            },
                            rule::tuple_pattern_target,
                        )
                    },
                );

                element.visit(visitor, Some((target, rule::tuple_pattern_element)));
            }

            (PlaceholderNode, rule::tuple_pattern)
        })
    }
}
