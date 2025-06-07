nodes! {
    mod annotate;
    mod apply;
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
    mod tuple;
    mod unit;
    mod name;
    mod when;
}

use crate::{Visit, Visitor};
use wipple_compiler_syntax::Expression;
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for Expression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        match self {
            Expression::Placeholder(expression) => expression.visit(visitor, parent),
            Expression::Name(expression) => expression.visit(visitor, parent),
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
            Expression::Annotate(expression) => expression.visit(visitor, parent),
            Expression::As(expression) => expression.visit(visitor, parent),
            Expression::To(expression) => expression.visit(visitor, parent),
            Expression::By(expression) => expression.visit(visitor, parent),
            Expression::Power(expression) => expression.visit(visitor, parent),
            Expression::Multiply(expression) => expression.visit(visitor, parent),
            Expression::Divide(expression) => expression.visit(visitor, parent),
            Expression::Remainder(expression) => expression.visit(visitor, parent),
            Expression::Add(expression) => expression.visit(visitor, parent),
            Expression::Subtract(expression) => expression.visit(visitor, parent),
            Expression::LessThan(expression) => expression.visit(visitor, parent),
            Expression::LessThanOrEqual(expression) => expression.visit(visitor, parent),
            Expression::GreaterThan(expression) => expression.visit(visitor, parent),
            Expression::GreaterThanOrEqual(expression) => expression.visit(visitor, parent),
            Expression::Equal(expression) => expression.visit(visitor, parent),
            Expression::NotEqual(expression) => expression.visit(visitor, parent),
            Expression::Is(expression) => expression.visit(visitor, parent),
            Expression::And(expression) => expression.visit(visitor, parent),
            Expression::Or(expression) => expression.visit(visitor, parent),
            Expression::Apply(expression) => expression.visit(visitor, parent),
            Expression::Tuple(expression) => expression.visit(visitor, parent),
            Expression::Collection(expression) => expression.visit(visitor, parent),
            Expression::Function(expression) => expression.visit(visitor, parent),
        }
    }
}
