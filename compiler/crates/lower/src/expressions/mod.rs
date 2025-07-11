mod annotate;
mod r#as;
mod binary;
mod block;
mod call;
mod collection;
mod r#do;
mod formatted_text;
mod function;
mod intrinsic;
mod r#is;
mod number;
mod placeholder;
mod structure;
mod text;
mod r#trait;
mod tuple;
mod unit;
mod variable;
mod when;

use crate::{Visit, Visitor};
use wipple_compiler_syntax::Expression;
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for Expression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        match self {
            Expression::Placeholder(expression) => expression.visit(visitor, parent),
            Expression::Variable(expression) => expression.visit(visitor, parent),
            Expression::Trait(expression) => expression.visit(visitor, parent),
            Expression::Number(expression) => expression.visit(visitor, parent),
            Expression::Text(expression) => expression.visit(visitor, parent),
            Expression::FormattedText(expression) => expression.visit(visitor, parent),
            Expression::Structure(expression) => expression.visit(visitor, parent),
            Expression::Block(expression) => expression.visit(visitor, parent),
            Expression::Unit(expression) => expression.visit(visitor, parent),
            Expression::Call(expression) => expression.visit(visitor, parent),
            Expression::Do(expression) => expression.visit(visitor, parent),
            Expression::When(expression) => expression.visit(visitor, parent),
            Expression::Intrinsic(expression) => expression.visit(visitor, parent),
            Expression::Binary(expression) => expression.visit(visitor, parent),
            Expression::Annotate(expression) => expression.visit(visitor, parent),
            Expression::As(expression) => expression.visit(visitor, parent),
            Expression::Is(expression) => expression.visit(visitor, parent),
            Expression::Tuple(expression) => expression.visit(visitor, parent),
            Expression::Collection(expression) => expression.visit(visitor, parent),
            Expression::Function(expression) => expression.visit(visitor, parent),
        }
    }
}
