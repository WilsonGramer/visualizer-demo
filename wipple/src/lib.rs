pub mod feedback;
pub mod matcher;
pub mod queries;
pub mod span;

pub use wipple_db as db;
pub use wipple_syntax as syntax;
pub use wipple_visit as visit;

use crate::{queries::run_query, span::ParsedSpan};
use colored::Colorize;
use db::{Db, Filter};
use line_index::LineIndex;
use std::io::Write;
use syntax::{Parse, Range};

pub fn run(
    path: &str,
    source: &str,
    filter: Option<Filter<'_>>,
    queries: impl IntoIterator<Item = (String, ParsedSpan)>,
    mut output: impl Write,
    graph: Option<impl Write>,
) -> anyhow::Result<()> {
    let source_file = match syntax::SourceFile::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            write!(output, "syntax error: {error}")?;
            return Ok(());
        }
    };

    let line_index = LineIndex::new(source);

    let mut db = Db::new();

    let info = visit::visit(&source_file, &mut db, |range: Range| {
        let Range::Some(start, end) = range else {
            panic!("node has no range");
        };

        let span = ParsedSpan::Range {
            path: path.to_string(),
            range: start..end,
        }
        .to_span(&line_index)
        .expect("invalid span");

        let source = source[start..end].to_string();

        (span, source)
    });

    let mut solver = visualizer::Solver::new(&mut db);
    solver.insert(info.definition_constraints);
    solver.insert(info.top_level_constraints);
    let ty_groups = solver.finish();

    for (query, span) in queries {
        let span = span
            .to_span(&line_index)
            .ok_or_else(|| anyhow::format_err!("invalid span: {span}"))?;

        let outputs = run_query(&query, &db, span)?;

        writeln!(
            output,
            "{}\n",
            format!("Result of query '{query}':").bold().underline()
        )?;

        if outputs.is_empty() {
            writeln!(output, "    no outputs")?;
        } else {
            for output_span in outputs {
                writeln!(
                    output,
                    "    {}: {}",
                    output_span,
                    source[output_span.range.clone()].blue()
                )?;
            }
        }

        writeln!(output)?;
    }

    feedback::write_feedback(&db, &mut output)?;

    writeln!(output, "{}\n", "Facts:".bold().underline())?;

    db.write(&ty_groups, filter, "  ", &mut output, graph)?;

    Ok(())
}
