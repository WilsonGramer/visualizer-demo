use crate::{Visit, Visitor};
use wipple_compiler_syntax::TuplePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{PlaceholderNode, TupleElementNode};


    /// A tuple pattern.
pub const TUPLE_PATTERN: Rule = Rule::new("tuple_pattern");

    /// The target of a tuple pattern.
pub const TUPLE_PATTERN_TARGET: Rule = Rule::new("tuple_pattern_target");

    /// An element in a tuple pattern.
pub const TUPLE_PATTERN_ELEMENT: Rule = Rule::new("tuple_pattern_element");


impl Visit for TuplePattern {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            for (index, element) in self.elements.iter().enumerate() {
                let target = visitor.node(
                    Some((id, TUPLE_PATTERN_TARGET)),
                    &self.range,
                    |visitor, _id| {
                        (
                            TupleElementNode {
                                index,
                                count: self.elements.len(),
                                target: visitor.parent(),
                            },
                            TUPLE_PATTERN_TARGET,
                        )
                    },
                );

                element.visit(visitor, Some((target, TUPLE_PATTERN_ELEMENT)));
            }

            (PlaceholderNode, TUPLE_PATTERN)
        })
    }
}
