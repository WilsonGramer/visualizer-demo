use crate::{Parse, Range, Rule};
use pest_ast::FromPest;

macro_rules! token {
    ($name:ident, $rule:ident $(, $f:ident)*) => {
        #[derive(Debug, Clone, PartialEq, FromPest)]
        #[pest_ast(rule(Rule::$rule))]
        pub struct $name {
            #[pest_ast(outer(with(Range::from)))]
            pub range: Range,

            #[pest_ast(outer(with(span_into_string) $(, with($f))*))]
            pub value: String,
        }

        impl Parse for $name {
            const RULE: crate::Rule = Rule::$rule;
        }
    };
}

token!(Comment, comment, trim_comment);
token!(Text, text, trim_quotes);
token!(Number, number);
token!(TypeName, type_name);
token!(VariantName, variant_name);
token!(VariableName, variable_name);
token!(TypeParameterName, type_parameter_name);
token!(AttributeName, attribute_name);

pub fn span_into_string(span: pest::Span) -> String {
    span.as_str().to_string()
}

fn trim_comment(s: String) -> String {
    s["--".len()..].to_string()
}

fn trim_quotes(s: String) -> String {
    s[1..s.len() - 1].to_string()
}
