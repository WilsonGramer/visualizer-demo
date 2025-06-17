mod annotate;
mod destructure;
mod number;
mod or;
mod set;
mod text;
mod tuple;
mod unit;
mod variable;
mod variant;
mod wildcard;

use crate::{Visit, Visitor};
use wipple_compiler_syntax::Pattern;
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for Pattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        match self {
            Pattern::Unit(pattern) => pattern.visit(visitor, parent),
            Pattern::Wildcard(pattern) => pattern.visit(visitor, parent),
            Pattern::Variable(pattern) => pattern.visit(visitor, parent),
            Pattern::Number(pattern) => pattern.visit(visitor, parent),
            Pattern::Text(pattern) => pattern.visit(visitor, parent),
            Pattern::Destructure(pattern) => pattern.visit(visitor, parent),
            Pattern::Set(pattern) => pattern.visit(visitor, parent),
            Pattern::Variant(pattern) => pattern.visit(visitor, parent),
            Pattern::Or(pattern) => pattern.visit(visitor, parent),
            Pattern::Tuple(pattern) => pattern.visit(visitor, parent),
            Pattern::Annotate(pattern) => pattern.visit(visitor, parent),
        }
    }
}
