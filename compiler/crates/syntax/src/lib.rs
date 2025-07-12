mod attributes;
mod constraints;
mod expressions;
mod patterns;
mod statements;
mod tokens;
mod types;

pub use attributes::*;
pub use constraints::*;
pub use expressions::*;
pub use patterns::*;
pub use statements::*;
pub use tokens::*;
pub use types::*;

use from_pest::FromPest;
use pest::Parser as _;
use pest_ast::FromPest;

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::source_file))]
pub struct SourceFile {
    #[pest_ast(outer(with(Range::from)))]
    pub range: Range,
    pub statements: Option<Statements>,
    _eoi: Eoi,
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct Eoi;

impl Parse for SourceFile {
    const RULE: crate::Rule = Rule::source_file;
}

#[derive(Debug, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::statements))]
pub struct Statements(pub Vec<Statement>);

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct Parser;

type Result<T> = std::result::Result<T, pest::error::Error<Rule>>;

#[derive(Debug, Clone, Copy)]
pub enum Range {
    None,
    Some(usize, usize),
}

impl<'pest> From<pest::Span<'pest>> for Range {
    fn from(span: pest::Span<'pest>) -> Self {
        Range::Some(span.start(), span.end())
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Range::Some(left_from, left_to), Range::Some(right_from, right_to)) => {
                left_from == right_from && left_to == right_to
            }
            _ => true,
        }
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Range::Some(left_from, left_to), Range::Some(right_from, right_to)) => {
                Some(left_from.cmp(right_from).then(left_to.cmp(right_to)))
            }
            _ => Some(std::cmp::Ordering::Equal),
        }
    }
}

pub trait Parse: for<'a> FromPest<'a, Rule = Rule, FatalError: std::fmt::Debug> {
    const RULE: Rule;

    #[allow(clippy::result_large_err)]
    fn parse(source: &str) -> Result<Self> {
        let mut pairs = Parser::parse(Self::RULE, source)?;

        #[cfg(test)]
        dbg!(&pairs);

        Ok(Self::from_pest(&mut pairs)
            .unwrap_or_else(|e| panic!("failed conversion from Pest: {e:?}")))
    }
}

// Parse variants without a wrapper rule
macro_rules! pest_enum {
    (
        #[parenthesized = $parenthesized:ty]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident($value:ty),)*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant($value),)*
        }

        impl<'pest> from_pest::FromPest<'pest> for $name {
            type Rule = crate::Rule;
            type FatalError = from_pest::Void;

            fn from_pest(
                pest: &mut pest::iterators::Pairs<'pest, Self::Rule>,
            ) -> Result<Self, from_pest::ConversionError<Self::FatalError>> {
                Err(from_pest::ConversionError::NoMatch::<Self::FatalError>)
                    $(.or_else(|_| <$value>::from_pest(pest).map(Self::$variant)))*
                    .or_else(|_| <$parenthesized>::from_pest(pest).map(|x| x.0))
            }
        }
    };
}

use pest_enum;

#[derive(Debug, Clone, PartialEq)]
struct NeverParenthesized<T>(T);

impl<'pest, T> FromPest<'pest> for NeverParenthesized<T> {
    type Rule = Rule;

    type FatalError = from_pest::Void;

    fn from_pest(
        _pest: &mut pest::iterators::Pairs<'pest, Self::Rule>,
    ) -> std::result::Result<Self, from_pest::ConversionError<Self::FatalError>> {
        Err(from_pest::ConversionError::NoMatch::<Self::FatalError>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file() {
        assert_eq!(
            SourceFile::parse("abc").unwrap(),
            SourceFile {
                range: Range::Some(0, 3),
                statements: Some(Statements(vec![Statement::Expression(
                    ExpressionStatement {
                        range: Range::None,
                        expression: Expression::Variable(VariableExpression {
                            range: Range::None,
                            variable: VariableName {
                                range: Range::Some(0, 3),
                                value: String::from("abc")
                            }
                        })
                    }
                )])),
                _eoi: Eoi,
            }
        );
    }
}
