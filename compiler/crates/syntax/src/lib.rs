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

use derive_tree_sitter::FromNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromNode)]
pub struct SourceFile {
    pub source: String,
    pub statements: Vec<Statement>,
}

pub type Result<T> = derive_tree_sitter::Result<T>;

pub fn parse(source: &str) -> Result<SourceFile> {
    derive_tree_sitter::parse(source, tree_sitter_wipple::LANGUAGE)
}
