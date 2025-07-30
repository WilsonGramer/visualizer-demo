use colored::Colorize;
use petgraph::Direction;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
};
use wasm_bindgen::prelude::*;
use wipple_compiler_lower::Definition;
use wipple_compiler_syntax::{Parse, Range};
use wipple_compiler_trace::{NodeId, Rule, Span};
use wipple_compiler_typecheck::{
    collect_constraints, debug,
    nodes::Node,
    typechecker::{TypeProvider, Typechecker},
};

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

#[derive(Debug, Clone)]
struct ClonedNode;

impl Node for ClonedNode {}

pub static UNKNOWN_TYPE: Rule = Rule::new("unknown type");
pub static INCOMPLETE_TYPE: Rule = Rule::new("incomplete type");
pub static UNRESOLVED_TRAIT: Rule = Rule::new("unresolved trait");
pub static RESOLVED_TRAIT: Rule = Rule::new("resolved trait");
pub static INSTANTIATED: Rule = Rule::new("instantiated");

pub fn compile(
    path: &str,
    source: &str,
    mut display_syntax: impl FnMut(String),
    mut display_graph: impl FnMut(String),
    mut display_tys: impl FnMut(String),
    mut display_feedback: impl FnMut(String),
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

    let mut extras = BTreeMap::<NodeId, HashSet<Rule>>::new();

    let constraints = collect_constraints(
        lowered
            .borrow()
            .nodes
            .iter()
            .map(|(&id, (node, _))| (id, node.as_ref())),
    );

    let type_provider = TypeProvider::new(
        |node_id| {
            let mut lowered = lowered.borrow_mut();

            let new_id = lowered.next_id;
            lowered.next_id.0 += 1;

            lowered
                .nodes
                .insert(new_id, (ClonedNode.boxed(), INSTANTIATED));

            if let Some(span) = lowered.spans.get(&node_id).cloned() {
                lowered.spans.insert(new_id, span);
            }

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
            lowered.relations.add_edge(node, bound.tr, RESOLVED_TRAIT);
            lowered.relations.add_edge(node, instance, RESOLVED_TRAIT);
        },
        |node, bound| {
            let mut lowered = lowered.borrow_mut();

            lowered.relations.add_edge(node, bound.tr, RESOLVED_TRAIT);

            // TODO: Show the trait being resolved, not the node, in feedback
            extras.entry(node).or_default().insert(UNRESOLVED_TRAIT);
        },
    );

    let mut typechecker = Typechecker::with_provider(type_provider);
    typechecker.insert_nodes(lowered.borrow().nodes.keys().copied());
    typechecker.insert_constraints(constraints);

    let ty_groups = typechecker.to_ty_groups();

    drop(typechecker);
    let lowered = lowered.into_inner();

    let filter = |node: NodeId| lowered.typed_nodes.contains(&node);

    // Ensure all expressions are typed (TODO: Put this in its own function)
    for &node in lowered.nodes.keys() {
        if !filter(node) {
            continue;
        }

        let tys = ty_groups
            .index_of(node)
            .map(|index| ty_groups.tys_at(index))
            .unwrap_or_default();

        if tys.is_empty() {
            extras.entry(node).or_default().insert(UNKNOWN_TYPE);
        } else if tys.iter().all(|ty| ty.is_incomplete()) {
            extras.entry(node).or_default().insert(INCOMPLETE_TYPE);
        }
    }

    let get_span_source = |node: NodeId| {
        let span = lowered.spans.get(&node).unwrap().clone();

        let mut source = source[span.range.clone()].to_string();

        // HACK: Remove comments
        source = source
            .lines()
            .skip_while(|line| line.starts_with("--"))
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

    let mut feedback_nodes = lowered
        .nodes
        .iter()
        .filter(|&(&node, _)| filter(node))
        .map(|(&node, &(_, rule))| (node, HashSet::from([rule])))
        .chain(ty_groups.nodes().map(|node| (node, HashSet::new())))
        .collect::<BTreeMap<_, _>>();

    for (node, rules) in extras {
        feedback_nodes.entry(node).or_default().extend(rules);
    }

    let provider = wipple_compiler_typecheck::feedback::FeedbackProvider::new(
        &feedback_nodes,
        &lowered.relations,
        get_span_source,
        get_comments,
    );

    // Display type graph

    let mut graph = String::new();
    debug::write_graph(
        &mut graph,
        &ty_groups,
        &lowered.relations,
        &provider,
        filter,
    )
    .unwrap();

    display_graph(graph);

    // Display feedback

    let feedback_ctx = wipple_compiler_feedback::Context::new(
        &provider,
        &feedback_nodes,
        &lowered.spans,
        &lowered.relations,
        &ty_groups,
    );

    let feedback = feedback_ctx.collect_feedback();

    display_feedback(
        feedback
            .into_iter()
            .filter_map(|(_, _, item, context)| item.render(&context, &provider))
            .collect::<String>(),
    );

    // Display type table

    let mut displayed_tys = Vec::from_iter(
        ty_groups
            .nodes()
            .chain(lowered.nodes.keys().copied())
            .collect::<BTreeSet<_>>(),
    );

    displayed_tys.sort_by_key(|node| {
        let span = lowered.spans.get(node).unwrap();
        (node.0, span.range.start, span.range.end)
    });

    let mut rows = Vec::new();

    for node in displayed_tys {
        if !filter(node) {
            continue;
        }

        let tys = ty_groups
            .index_of(node)
            .map(|index| ty_groups.tys_at(index))
            .unwrap_or_default();

        let (node_span, node_debug) = provider.node_span_source(node);

        let node_related_rules = lowered
            .relations
            .neighbors_directed(node, Direction::Incoming)
            .filter(|&node| filter(node))
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
                .map(|ty| ty.to_debug_string(&provider).blue().to_string())
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
