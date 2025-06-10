use colored::Colorize;
use std::{collections::BTreeMap, sync::Arc};
use wasm_bindgen::prelude::*;
use wipple_compiler_trace::{AnyRule, NodeId, Rule, RuleKind, Span, rule};

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

rule! {
    /// A node's type is unknown.
    unknown_type: Extra;

    /// A node's type is incomplete.
    incomplete_type: Extra;
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

    let groups = typecheck_session.groups(None);
    let mut tys = typecheck_session.iterate(groups);

    // Also include the relations gathered during lowering
    for (&node, related) in &lowered.relations {
        tys.entry(node)
            .or_default()
            .1
            .extend(related.iter().copied());
    }

    // Ensure all expressions are typed (TODO: Put this in its own function)
    let mut extras = BTreeMap::<NodeId, Vec<AnyRule>>::new();
    for (&node, &(_, rule)) in &lowered.nodes {
        if rule.kind() != RuleKind::Typed {
            continue;
        }

        if let Some((tys, _)) = tys.get(&node) {
            for ty in tys {
                let mut incomplete = false;
                ty.traverse(&mut |ty| {
                    if ty.is_unknown_shallow() {
                        incomplete = true;
                    }
                });

                if incomplete {
                    extras
                        .entry(node)
                        .or_default()
                        .push(rule::incomplete_type.erased());
                }
            }
        } else {
            extras
                .entry(node)
                .or_default()
                .push(rule::unknown_type.erased());
        }
    }

    let provider = wipple_compiler_typecheck::context::FeedbackProvider::new(&tys, |node| {
        let Some(span) = lowered.spans.get(&node) else {
            return (Span::root(path.clone()), String::from("<unknown>"));
        };

        let source = &source[span.range.clone()];

        (span.clone(), source.to_string())
    });

    // Display feedback

    let feedback_nodes = lowered
        .nodes
        .iter()
        .map(|(&id, &(_, rule))| {
            let extra = extras.get(&id).cloned().unwrap_or_default();
            (id, extra.into_iter().chain([rule]).collect())
        })
        .collect();

    let feedback_ctx = wipple_compiler_feedback::Context::new(
        &feedback_nodes,
        &lowered.spans,
        &lowered.names,
        &lowered.relations,
        &tys,
    );

    let feedback = feedback_ctx.collect_feedback();

    display_feedback(
        feedback
            .into_iter()
            .filter_map(|(_, item, context)| item.render(&context, &provider))
            .collect::<String>(),
    );

    // Display type graph

    let graph = typecheck_session.to_debug_graph(None, &tys, &lowered.relations, &provider);
    display_graph(graph);

    // Display type table

    let mut displayed_tys = Vec::from_iter(&tys);
    displayed_tys.sort_by_key(|(node, _)| {
        let span = lowered.spans.get(node).unwrap();
        (span.range.start, span.range.end)
    });

    let mut rows = Vec::new();

    for (&node, (tys, related)) in displayed_tys {
        let (node_span, node_debug) = provider.node_span_source(node);

        let related_rules = related
            .iter()
            .map(|(&node, &rule)| {
                format!("\n  via {:?}: {}", rule, provider.node_span_source(node).1)
            })
            .collect::<String>();

        rows.push([
            format!("{node_span:?}").dimmed().to_string(),
            format!("{:?}", lowered.nodes.get(&node).unwrap().1),
            node_debug.to_string(),
            tys.iter()
                .map(|ty| ty.to_debug_string(&provider).blue().to_string())
                .collect::<Vec<_>>()
                .join(&" or ".bright_red().to_string())
                + &related_rules,
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
