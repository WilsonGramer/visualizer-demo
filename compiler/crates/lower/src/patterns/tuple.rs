use crate::{Visit, Visitor};
use wipple_compiler_syntax::TuplePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::TupleNode;

pub static TUPLE_PATTERN: Rule = Rule::new("tuple pattern");
pub static TUPLE_PATTERN_TARGET: Rule = Rule::new("tuple pattern target");
pub static TUPLE_PATTERN_ELEMENT: Rule = Rule::new("tuple pattern element");

impl Visit for TuplePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let elements = self
                .elements
                .iter()
                .map(|element| element.visit(visitor, (id, TUPLE_PATTERN_ELEMENT)))
                .collect::<Vec<_>>();

            (TupleNode { elements }, TUPLE_PATTERN)
        })
    }
}
