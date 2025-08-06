pub mod attributes;
pub mod constraints;
pub mod definitions;
pub mod nodes;
pub mod visitor;

use crate::visitor::{ProgramInfo, Visitor};
use wipple_db::{Db, Span};
use wipple_syntax::{Range, SourceFile, Statement};

pub fn visit(
    file: &SourceFile,
    db: &mut Db,
    get_span_source: impl Fn(Range) -> (Span, String),
) -> ProgramInfo {
    let mut visitor = Visitor::new(db, get_span_source);

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
