use crate::{Visit, Visitor};
use wipple_compiler_syntax::TuplePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{PlaceholderNode, TupleElementNode};

pub const TUPLE_PATTERN: Rule = Rule::new("tuple pattern");

pub const TUPLE_PATTERN_TARGET: Rule = Rule::new("tuple pattern target");

pub const TUPLE_PATTERN_ELEMENT: Rule = Rule::new("tuple pattern element");

impl Visit for TuplePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            for (index, element) in self.elements.iter().enumerate() {
                let target =
                    visitor.node((id, TUPLE_PATTERN_TARGET), &self.range, |visitor, _id| {
                        (
                            TupleElementNode {
                                index,
                                count: self.elements.len(),
                                target: visitor.target(),
                            },
                            TUPLE_PATTERN_TARGET,
                        )
                    });

                visitor.with_target(target, |visitor| {
                    element.visit(visitor, (id, TUPLE_PATTERN_ELEMENT))
                });
            }

            (PlaceholderNode, TUPLE_PATTERN)
        })
    }
}
