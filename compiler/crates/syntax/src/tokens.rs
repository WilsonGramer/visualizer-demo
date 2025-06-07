use derive_tree_sitter::FromNode;
use std::ops::Range;

macro_rules! tokens {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
            pub struct $name {
                pub range: Range<usize>,
                pub source: String,
            }
        )*
    }
}

tokens!(
    Comment,
    Text,
    Number,
    TypeName,
    VariantName,
    VariableName,
    TypeParameterName,
    AttributeName,
    BinaryOperator,
);
