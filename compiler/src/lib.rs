use colored::Colorize;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wipple_compiler_trace::Span;
use wipple_compiler_typecheck::context::DebugOptions;

#[wasm_bindgen(js_name = "compile")]
pub fn compile_wasm(source: String) -> Vec<String> {
    console_error_panic_hook::set_once();

    let mut output_syntax_error = String::new();
    let mut output_graph = String::new();
    let mut output_tys = String::new();
    let mut output_feedback = String::new();

    compile(
        "input",
        &source,
        |error| output_syntax_error.push_str(&error),
        |graph| output_graph.push_str(&graph),
        |tys| output_tys.push_str(&tys),
        |feedback| output_feedback.push_str(&feedback),
    );

    vec![
        output_syntax_error,
        output_graph,
        output_tys,
        output_feedback,
    ]
}

pub fn compile(
    path: &str,
    source: &str,
    mut display_syntax_error: impl FnMut(String),
    mut display_graph: impl FnMut(String),
    mut display_tys: impl FnMut(String),
    mut display_feedback: impl FnMut(String),
) {
    let source_file = match wipple_compiler_syntax::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            display_syntax_error(format!("{error:?}"));
            return;
        }
    };

    let path = Arc::<str>::from(path);
    let line_col = line_col::LineColLookup::new(source);

    let lowered = wipple_compiler_lower::visit(&source_file, |range| Span {
        path: path.clone(),
        range: range.clone(),
        start_line_col: line_col.get(range.start),
        end_line_col: line_col.get(range.end),
    });

    let typecheck_ctx = wipple_compiler_typecheck::context::Context {
        nodes: lowered
            .nodes
            .iter()
            .map(|(&id, (node, _))| (id, node.as_ref()))
            .collect(),
    };

    let mut typecheck_session = typecheck_ctx.session();

    let tys = typecheck_session.iterate();

    let debug = wipple_compiler_typecheck::context::DebugProvider::new(&tys, |node, options| {
        let Some(span) = lowered.spans.get(&node) else {
            return (Span::root(path.clone()), String::from("<unknown>"));
        };

        let source = &source[span.range.clone()];

        let result = if options.rule {
            let rule = lowered
                .nodes
                .get(&node)
                .map(|(_, rule)| format!("{rule:?}"))
                .unwrap_or_else(|| String::from("<unknown>"));

            format!("{source}\n{rule}")
        } else {
            source.to_string()
        };

        (span.clone(), result)
    });

    // Display feedback

    let feedback_nodes = lowered
        .nodes
        .iter()
        .map(|(&id, &(_, rule))| (id, rule))
        .collect();

    let feedback_ctx = wipple_compiler_feedback::Context::new(
        &feedback_nodes,
        &lowered.spans,
        &lowered.relations,
        &tys,
    );

    let feedback = feedback_ctx.collect_feedback();

    display_feedback(
        feedback
            .into_iter()
            .map(|feedback| feedback.to_markdown(&debug))
            .collect::<String>(),
    );

    // Display type graph

    let graph = typecheck_session.to_debug_graph(None, &tys, &lowered.relations, &debug);
    display_graph(graph);

    // Display type table

    let mut displayed_tys = Vec::from_iter(&tys);
    displayed_tys.sort_by_key(|(node, _)| {
        let span = lowered.spans.get(node).unwrap();
        (span.range.start, span.range.end)
    });

    let mut rows = Vec::new();

    for (&node, tys) in displayed_tys {
        let (node_span, node_debug) = debug.node_source(node, DebugOptions::default());

        rows.push([
            format!("{node_span:?}").dimmed().to_string(),
            node_debug.to_string(),
            tys.iter()
                .map(|ty| ty.to_debug_string(&debug).blue().to_string())
                .collect::<Vec<_>>()
                .join(&" or ".bright_red().to_string()),
            format!("{:?}", lowered.nodes.get(&node).unwrap().1),
        ]);
    }

    rows.dedup();

    let mut table = tabled::builder::Builder::new();
    table.push_record(["Span", "Node", "Type", "Rule"]);
    for row in rows {
        table.push_record(row);
    }

    display_tys(format!(
        "{}",
        table
            .build()
            .with(tabled::settings::Style::sharp().line_horizontal(
                tabled::settings::style::HorizontalLine::inherit(tabled::settings::Style::modern())
            ))
    ));
}
