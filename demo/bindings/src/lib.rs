use itertools::Itertools;
use std::io::{self, Write};
use visualizer::db::{Db, Span};
use visualizer::typecheck::Solver;
use visualizer::{Filter, visualize};
use wasm_bindgen::prelude::*;
use wipple_syntax::{Parse, Range};

#[wasm_bindgen(js_name = "run")]
pub fn run_wasm(source: String, filter: Option<Vec<u32>>) -> Vec<String> {
    console_error_panic_hook::set_once();
    colored::control::set_override(true);

    let filter = filter
        .and_then(|filter| filter.into_iter().collect_tuple())
        .map(|(start, end)| Filter::Range(start, end));

    let mut output = Vec::new();
    let mut graph = Vec::new();
    run("input", &source, filter, &mut output, Some(&mut graph)).unwrap();

    vec![
        String::from_utf8(output).unwrap(),
        String::from_utf8(graph).unwrap(),
    ]
}

pub fn run(
    path: &str,
    source: &str,
    filter: Option<Filter<'_>>,
    mut output: impl Write,
    graph: Option<impl Write>,
) -> io::Result<()> {
    let source_file = match wipple_syntax::SourceFile::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            write!(output, "syntax error: {error}")?;
            return Ok(());
        }
    };

    let line_col = line_col::LineColLookup::new(source);

    let mut db = Db::new();

    let info = wipple_visit::visit(&source_file, &mut db, |range: Range| {
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

    let mut solver = Solver::new(&mut db);
    solver.insert(info.definition_constraints);
    solver.insert(info.top_level_constraints);
    let ty_groups = solver.finish();

    visualize(&db, &ty_groups, filter, output, graph)?;

    Ok(())
}
