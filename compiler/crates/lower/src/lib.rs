pub mod attributes;
pub mod constraints;
pub mod definitions;
pub mod nodes;
pub mod visitor;

use crate::visitor::{Result, Visitor};
use wipple_visualizer_syntax::{Range, SourceFile, Statement};
use wipple_visualizer_typecheck::Span;

pub fn visit(file: &SourceFile, make_span: impl Fn(Range) -> Span) -> Result {
    let mut visitor = Visitor::new(make_span);

    let source_file = visitor.node(file.range, "sourceFile");
    visitor.hide(source_file);

    if let Some(statements) = &file.statements {
        for statement in &statements.0 {
            if !matches!(statement, Statement::Empty(_)) {
                visitor.child(statement, source_file, "statementInSourceFile");
            }
        }
    }

    visitor.finish()
}
