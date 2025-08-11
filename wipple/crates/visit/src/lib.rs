pub mod attributes;
pub mod constraints;
pub mod definitions;
pub mod nodes;
pub mod visitor;

use crate::visitor::{ProgramInfo, Visitor};
use wipple_syntax::{SourceFile, Statement};

pub use crate::visitor::Ctx;

pub fn visit(file: &SourceFile, ctx: Ctx<'_>) -> ProgramInfo {
    let mut visitor = Visitor::new(ctx);

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
