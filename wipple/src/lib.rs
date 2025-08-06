pub use wipple_db as db;
pub use wipple_syntax as syntax;
pub use wipple_visit as visit;

use db::{Db, Filter, Span};
use std::io::{self, Write};
use syntax::{Parse, Range};

pub fn run(
    path: &str,
    source: &str,
    filter: Option<Filter<'_>>,
    mut output: impl Write,
    graph: Option<impl Write>,
) -> io::Result<()> {
    let source_file = match syntax::SourceFile::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            write!(output, "syntax error: {error}")?;
            return Ok(());
        }
    };

    let line_col = line_col::LineColLookup::new(source);

    let mut db = Db::new();

    let info = visit::visit(&source_file, &mut db, |range: Range| {
        let Range::Some(start, end) = range else {
            panic!("node has no range");
        };

        let span = Span {
            path: path.to_string(),
            range: start..end,
            start_line_col: line_col.get(start),
            end_line_col: line_col.get(end),
        };

        let source = source[start..end].to_string();

        (span, source)
    });

    let mut solver = visualizer::Solver::new(&mut db);
    solver.insert(info.definition_constraints);
    solver.insert(info.top_level_constraints);
    let ty_groups = solver.finish();

    db.write(&ty_groups, filter, output, graph)?;

    Ok(())
}
