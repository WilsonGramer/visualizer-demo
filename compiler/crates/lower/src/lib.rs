pub mod attributes;
pub mod constraints;
pub mod definitions;
pub mod nodes;
pub mod visitor;

use crate::visitor::{Result, Visitor};
use wipple_compiler_syntax::{Range, SourceFile, Statement};
use wipple_compiler_trace::Span;

pub fn visit(file: &SourceFile, make_span: impl Fn(Range) -> Span) -> Result {
    let mut visitor = Visitor::new(make_span);

    let source_file = visitor.node(file.range, "source file [ignore]");
    if let Some(statements) = &file.statements {
        for statement in &statements.0 {
            if !matches!(statement, Statement::Empty(_)) {
                visitor.child(statement, source_file, "statement in source file");
            }
        }
    }

    visitor.finish()
}
