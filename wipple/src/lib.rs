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
use visualizer::Graph;

#[derive(Default)]
pub struct Options<'a> {
    pub path: &'a str,
    pub source: &'a str,
    pub filter: Vec<Filter<'a>>,
    pub queries: Vec<(String, ParsedSpan)>,
}

pub fn run(
    options: Options<'_>,
    mut output: impl Write,
    graph: Option<impl FnOnce(Graph)>,
) -> anyhow::Result<()> {
    let source_file = match syntax::SourceFile::parse(options.source) {
        Ok(source_file) => source_file,
        Err(error) => {
            write!(output, "syntax error: {error}")?;
            return Ok(());
        }
    };

    let line_index = LineIndex::new(options.source);

    let mut db = Db::new();

    let ctx = visit::Ctx {
        db: &mut db,
        get_span_source: Box::new(|range: Range| {
            let Range::Some(start, end) = range else {
                panic!("node has no range");
            };

            let span = ParsedSpan::Range {
                path: options.path.to_string(),
                range: start..end,
            }
            .to_span(&line_index)
            .expect("invalid span");

            let source = options.source[start..end].to_string();

            (span, source)
        }),
        show_definitions: true, // TODO: make this an option?
    };

    let info = visit::visit(&source_file, ctx);

    let mut solver = visualizer::Solver::new(&mut db);
    solver.insert(info.constraints);
    let ty_groups = solver.finish();

    for (query, span) in options.queries {
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
                    options.source[output_span.range.clone()].blue()
                )?;
            }
        }

        writeln!(output)?;
    }

    feedback::write_feedback(&db, &mut output)?;

    writeln!(output, "{}\n", "Facts:".bold().underline())?;

    db.write(&options.filter, "  ", &mut output)?;

    if let Some(graph) = graph {
        graph(db.graph(&ty_groups, &options.filter));
    }

    Ok(())
}
