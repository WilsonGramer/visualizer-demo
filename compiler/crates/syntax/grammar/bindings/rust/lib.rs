use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_wipple() -> *const ();
}

pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_wipple) };
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");
