use crate::{Visit, Visitor};
use wipple_compiler_syntax::Constraint;
use wipple_compiler_trace::{NodeId, Rule};

mod bound;
mod default;
mod infer;

impl Visit for Constraint {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        match self {
            Constraint::Bound(statement) => statement.visit(visitor, parent),
            Constraint::Infer(statement) => statement.visit(visitor, parent),
            Constraint::Default(statement) => statement.visit(visitor, parent),
        }
    }
}
