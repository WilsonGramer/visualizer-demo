use colored::Colorize;
use std::{cell::RefCell, collections::BTreeSet};
use wasm_bindgen::prelude::*;
use wipple_compiler_lower::definitions::Definition;
use wipple_compiler_syntax::{Parse, Range};
use wipple_compiler_trace::{Fact, NodeId, Span};
use wipple_compiler_typecheck::{TypeProvider, Typechecker, debug};

#[wasm_bindgen(js_name = "compile")]
pub fn compile_wasm(source: String) -> Vec<String> {
    console_error_panic_hook::set_once();

    let mut output_syntax_error = String::new();
    let mut output_graph = String::new();
    let mut output_tys = String::new();

    compile(
        "input",
        &source,
        |error| output_syntax_error.push_str(&error),
        |graph| output_graph.push_str(&graph),
        |tys| output_tys.push_str(&tys),
    );

    vec![output_syntax_error, output_graph, output_tys]
}

pub fn compile(
    path: &str,
    source: &str,
    mut display_syntax: impl FnMut(String),
    mut display_graph: impl FnMut(String),
    mut display_tys: impl FnMut(String),
) {
    let source_file = match wipple_compiler_syntax::SourceFile::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            display_syntax(format!("syntax error: {error}"));
            return;
        }
    };

    let line_col = line_col::LineColLookup::new(source);

    let make_span = |range: Range| {
        let Range::Some(start, end) = range else {
            panic!("node has no range");
        };

        Span {
            path: path.to_string(),
            range: start..end,
            start_line_col: line_col.get(start),
            end_line_col: line_col.get(end),
        }
    };

    let lowered = RefCell::new(wipple_compiler_lower::visit(&source_file, make_span));

    let type_provider = TypeProvider::new(
        |node_id| {
            let mut lowered = lowered.borrow_mut();

            let new_id = lowered.next_id;
            lowered.next_id.0 += 1;

            let span = lowered.spans.get(&node_id).unwrap().clone();
            lowered.spans.insert(new_id, span);

            new_id
        },
        |trait_id| {
            lowered
                .borrow()
                .instances
                .get(&trait_id)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|node| {
                    let lowered = lowered.borrow();
                    let definition = lowered.definitions.get(&node).unwrap();

                    let Definition::Instance(instance) = definition else {
                        unreachable!()
                    };

                    (node, instance.substitutions.clone())
                })
                .collect()
        },
        |node, bound, instance| {
            let mut lowered = lowered.borrow_mut();

            lowered.typed_nodes.insert(node);

            let facts = lowered.facts.entry(node).or_default();
            facts.insert(Fact::with_node(bound.tr, "resolved trait"));
            facts.insert(Fact::with_node(instance, "resolved trait"));
        },
        |node, bound| {
            let mut lowered = lowered.borrow_mut();

            lowered.typed_nodes.insert(node);

            lowered
                .facts
                .entry(node)
                .or_default()
                .insert(Fact::with_node(bound.tr, "unresolved trait"));
        },
    );

    let mut typechecker = Typechecker::with_provider(type_provider);
    {
        let nodes = lowered.borrow().facts.keys().cloned().collect::<Vec<_>>();
        let constraints = lowered.borrow().constraints.clone();
        typechecker.insert_nodes(nodes);
        typechecker.insert_constraints(constraints);
    }

    let ty_groups = typechecker.to_ty_groups();

    drop(typechecker);
    let mut lowered = lowered.into_inner();

    // Ensure all expressions are typed (TODO: Put this in its own function)
    for &node in lowered.typed_nodes.iter() {
        if lowered
            .facts
            .get(&node)
            .is_some_and(|facts| facts.iter().any(Fact::should_ignore))
        {
            continue;
        }

        let tys = ty_groups
            .index_of(node)
            .map(|index| ty_groups.tys_at(index))
            .unwrap_or_default();

        if tys.is_empty() {
            lowered
                .facts
                .entry(node)
                .or_default()
                .insert(Fact::marker("unknownType"));
        } else if tys.iter().all(|ty| ty.is_incomplete()) {
            lowered
                .facts
                .entry(node)
                .or_default()
                .insert(Fact::marker("incompleteType"));
        }
    }

    let get_span_source = |node: NodeId| {
        let span = lowered.spans.get(&node).unwrap().clone();

        let mut source = source[span.range.clone()].to_string();

        // HACK: Remove comments
        source = source
            .lines()
            .skip_while(|line| line.is_empty() || line.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        (span, source)
    };

    let get_comments = |node| {
        lowered
            .definitions
            .get(&node)
            .and_then(|definition| definition.comments())
            .map(|comments| {
                comments
                    .0
                    .iter()
                    .map(|comment| comment.value.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    };

    let provider = wipple_compiler_typecheck::feedback::FeedbackProvider::new(
        &lowered.facts,
        get_span_source,
        get_comments,
    );

    // Display type graph

    let mut graph = String::new();
    debug::write_graph(&mut graph, &ty_groups, &lowered.facts, &provider).unwrap();

    display_graph(graph);

    // Display type table

    let mut displayed_tys = Vec::from_iter(
        ty_groups
            .nodes()
            .chain(lowered.facts.keys().copied())
            .collect::<BTreeSet<_>>(),
    );

    displayed_tys.sort_by_key(|node| {
        let span = lowered.spans.get(node).unwrap();
        (node.0, span.range.start, span.range.end)
    });

    let mut rows = Vec::new();
    for node in displayed_tys {
        let (node_span, node_debug) = provider.node_span_source(node);

        let facts = lowered.facts.get(&node);

        if facts.is_some_and(|facts| facts.iter().any(Fact::should_ignore)) {
            continue;
        }

        let node_facts = facts
            .map(|facts| {
                facts
                    .iter()
                    .map(|fact| fact.to_string())
                    .collect::<Vec<_>>()
                    .join(",\n")
            })
            .unwrap_or_default();

        let tys = ty_groups
            .index_of(node)
            .map(|index| ty_groups.tys_at(index))
            .unwrap_or_default();

        rows.push([
            format!("{node:?}\n{node_span:?}").to_string(),
            node_facts,
            node_debug.to_string(),
            tys.iter()
                .map(|ty| ty.to_debug_string(&provider).blue().to_string())
                .collect::<Vec<_>>()
                .join(&" or ".bright_red().to_string()),
        ]);
    }

    if !rows.is_empty() {
        let mut table = tabled::builder::Builder::new();
        table.push_record(["Span", "Fact", "Node", "Type"]);
        for row in rows {
            table.push_record(row);
        }

        let width = 30;

        display_tys(format!(
            "{}",
            table
                .build()
                .with(tabled::settings::Style::sharp().line_horizontal(
                    tabled::settings::style::HorizontalLine::inherit(
                        tabled::settings::Style::modern()
                    )
                ))
                .with(
                    tabled::settings::Modify::new(tabled::settings::object::Segment::all())
                        .with(tabled::settings::Width::wrap(width))
                        .with(tabled::settings::Width::increase(width))
                )
        ));
    }
}
