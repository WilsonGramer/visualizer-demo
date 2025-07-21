use colored::Colorize;
use petgraph::{Direction, prelude::DiGraphMap};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use wasm_bindgen::prelude::*;
use wipple_compiler_lower::Definition;
use wipple_compiler_syntax::{Parse, Range};
use wipple_compiler_trace::{NodeId, Rule, Span};
use wipple_compiler_typecheck::{
    debug,
    id::TypedNodeId,
    typechecker::{TypeProvider, Typechecker, UNRESOLVED_TRAIT},
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

pub static UNKNOWN_TYPE: Rule = Rule::new("unknown type");
pub static INCOMPLETE_TYPE: Rule = Rule::new("incomplete type");
pub static RESOLVED_TRAIT: Rule = Rule::new("resolved trait");

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

    let lowered = wipple_compiler_lower::visit(&source_file, make_span);

    let mut extras = BTreeMap::<TypedNodeId, HashSet<Rule>>::new();
    let mut relations = lowered
        .relations
        .all_edges()
        .map(|(from, to, rule)| (from.into(), to.into(), rule))
        .collect::<DiGraphMap<_, _>>();

    let typecheck_ctx = wipple_compiler_typecheck::context::Context {
        nodes: lowered
            .nodes
            .iter()
            .map(|(&id, (node, rule))| (id, (node.as_ref(), *rule)))
            .collect(),
    };

    let constraints = typecheck_ctx.as_constraints();

    let type_provider = TypeProvider::new(
        |trait_id| {
            lowered
                .instances
                .get(&trait_id)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|node| {
                    let definition = lowered.definitions.get(&node).unwrap();

                    let Definition::Instance(instance) = definition else {
                        unreachable!()
                    };

                    (node, instance.substitutions.clone(), RESOLVED_TRAIT)
                })
                .collect()
        },
        |node, bound, instance| {
            relations.add_edge(bound.instantiation.into(), node, RESOLVED_TRAIT);
            relations.add_edge(node, instance.into(), RESOLVED_TRAIT);
        },
        |node, bound| {
            // TODO: Show the trait being resolved, not the node

            extras
                .entry(bound.instantiation.into())
                .or_default()
                .insert(UNRESOLVED_TRAIT);
        },
    );

    let mut typechecker = Typechecker::with_provider(type_provider);
    typechecker.insert_nodes(lowered.typed_nodes.clone());
    typechecker.insert_tys(&constraints.tys);
    typechecker.insert_bounds(&constraints.bounds);

    let ty_groups = typechecker.to_ty_groups();

    drop(typechecker);

    // Ensure all expressions are typed (TODO: Put this in its own function)
    for &node in &lowered.typed_nodes {
        let tys = ty_groups
            .index_of(node.into())
            .map(|index| ty_groups.tys_at(index))
            .unwrap_or_default();

        if tys.is_empty() {
            extras.entry(node.into()).or_default().insert(UNKNOWN_TYPE);
        } else if tys.iter().all(|ty| ty.is_incomplete()) {
            extras
                .entry(node.into())
                .or_default()
                .insert(INCOMPLETE_TYPE);
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
        .map(|(&node, &(_, rule))| (node.into(), HashSet::from([rule])))
        .chain(ty_groups.nodes().map(|node| (node, HashSet::new())))
        .collect::<BTreeMap<_, _>>();

    for (node, rules) in extras {
        feedback_nodes.entry(node).or_default().extend(rules);
    }

    let provider = wipple_compiler_typecheck::context::FeedbackProvider::new(
        &feedback_nodes,
        &lowered.relations,
        get_span_source,
        get_comments,
    );

    // Display type graph
    let filter = |node: TypedNodeId| lowered.typed_nodes.contains(&node.untyped());

    let mut graph = String::new();
    debug::write_graph(&mut graph, &ty_groups, &relations, &provider, filter).unwrap();

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
            .chain(lowered.nodes.keys().copied().map(TypedNodeId::from))
            .collect::<BTreeSet<_>>(),
    );

    displayed_tys.sort_by_key(|node| {
        let span = lowered.spans.get(&node.untyped()).unwrap();
        (node.instantiation, span.range.start, span.range.end)
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

        let (node_span, node_debug) = provider.node_span_source(node.untyped());

        let node_related_rules = lowered
            .relations
            .neighbors_directed(node.untyped(), Direction::Incoming)
            .filter(|node| lowered.typed_nodes.contains(node))
            .map(|related| {
                let rule = lowered
                    .relations
                    .edge_weight(related, node.untyped())
                    .unwrap();

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
                lowered.nodes.get(&node.untyped()).unwrap().1,
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
