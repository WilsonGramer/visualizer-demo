use colored::Colorize;
use petgraph::Direction;
use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};
use wasm_bindgen::prelude::*;
use wipple_compiler_trace::{NodeId, Rule, Span};

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

pub const UNKNOWN_TYPE: Rule = Rule::new("unknown type");

pub const INCOMPLETE_TYPE: Rule = Rule::new("incomplete type");

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

    let line_col = line_col::LineColLookup::new(source);

    let make_span = |range: Range<usize>| Span {
        path: path.to_string(),
        range: range.clone(),
        start_line_col: line_col.get(range.start),
        end_line_col: line_col.get(range.end),
    };

    let lowered = wipple_compiler_lower::visit(&source_file, make_span);

    let feedback_nodes = lowered
        .nodes
        .iter()
        .map(|(&id, &(_, rule))| (id, HashSet::from([rule])))
        .collect::<BTreeMap<_, _>>();

    let get_span_source = |node| {
        let span = lowered.spans.get(&node).unwrap();
        let source = &source[span.range.clone()];
        (span.clone(), source.to_string())
    };

    let provider = wipple_compiler_typecheck::context::FeedbackProvider::new(
        &feedback_nodes,
        &lowered.relations,
        get_span_source,
    );

    let typecheck_ctx = wipple_compiler_typecheck::context::Context {
        nodes: lowered
            .nodes
            .iter()
            .map(|(&id, (node, rule))| (id, (node.as_ref(), *rule)))
            .collect(),
    };

    for &node in lowered.nodes.keys() {
        let (span, source) = provider.node_span_source(node);
        eprint!("{node:?} ==> {span:?}: {source}");

        if lowered.typed_nodes.contains(&node) {
            eprintln!(" (typed)");
        }

        eprintln!();
    }

    let filter = |node| lowered.typed_nodes.contains(&node);

    let constraints = typecheck_ctx.typechecker_from_constraints_by(filter).run();

    let mut buf = Vec::new();
    constraints
        .write_debug_graph(&mut buf, &lowered.relations, &provider, filter)
        .unwrap();

    display_graph(String::from_utf8(buf).unwrap());

    // Ensure all expressions are typed (TODO: Put this in its own function)
    let mut extras = BTreeMap::<NodeId, HashSet<Rule>>::new();
    for &node in lowered.nodes.keys() {
        if !lowered.typed_nodes.contains(&node) {
            continue;
        }

        if let Some(tys) = constraints.tys.get(&node) {
            for (ty, _) in tys {
                if ty.is_incomplete() {
                    extras.entry(node).or_default().insert(INCOMPLETE_TYPE);
                }
            }
        } else {
            extras.entry(node).or_default().insert(UNKNOWN_TYPE);
        }
    }

    // Display feedback

    let feedback_ctx = wipple_compiler_feedback::Context::new(
        &provider,
        &feedback_nodes,
        &lowered.spans,
        &lowered.relations,
        &constraints.tys,
    );

    let feedback = feedback_ctx.collect_feedback();

    display_feedback(
        feedback
            .into_iter()
            .filter_map(|(_, _, item, context)| item.render(&context, &provider))
            .collect::<String>(),
    );

    // Display type table

    let mut displayed_tys = Vec::from_iter(&constraints.tys);
    displayed_tys.sort_by_key(|(node, _)| {
        let span = lowered.spans.get(node).unwrap();
        (span.range.start, span.range.end)
    });

    let mut rows = Vec::new();

    for (&node, tys) in displayed_tys {
        if !lowered.typed_nodes.contains(&node) {
            continue;
        }

        let (node_span, node_debug) = provider.node_span_source(node);

        let node_related_rules = lowered
            .relations
            .neighbors_directed(node, Direction::Incoming)
            .filter(|node| lowered.typed_nodes.contains(node))
            .map(|related| {
                let rule = lowered.relations.edge_weight(related, node).unwrap();
                format!(
                    "\n  via {:?}: {}",
                    rule,
                    provider.node_span_source(related).1
                )
            })
            .collect::<String>();

        rows.push([
            format!("{node:?}\n{node_span:?}").to_string(),
            format!(
                "{:?}{}",
                lowered.nodes.get(&node).unwrap().1,
                node_related_rules
            ),
            node_debug.to_string(),
            tys.iter()
                .map(|(ty, _)| ty.to_debug_string(&provider).blue().to_string())
                .collect::<Vec<_>>()
                .join(&" or ".bright_red().to_string()),
        ]);
    }

    rows.dedup();

    if !rows.is_empty() {
        let mut table = tabled::builder::Builder::new();
        table.push_record(["Span", "Rule", "Node", "Type"]);
        for row in rows {
            table.push_record(row);
        }

        display_tys(format!(
            "{}",
            table
                .build()
                .with(tabled::settings::Style::sharp().line_horizontal(
                    tabled::settings::style::HorizontalLine::inherit(
                        tabled::settings::Style::modern()
                    )
                ))
        ));
    }
}
