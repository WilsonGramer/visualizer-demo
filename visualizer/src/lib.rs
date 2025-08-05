mod display;

use std::{
    io::{self, Write},
    mem,
};
use wasm_bindgen::prelude::*;
use wipple_visualizer_lower::definitions::Definition;
use wipple_visualizer_syntax::{Parse, Range};
use wipple_visualizer_typecheck::{Fact, NodeId, Span, Ty, Typechecker};

#[wasm_bindgen(js_name = "compile")]
pub fn compile_wasm(source: String) -> Vec<String> {
    console_error_panic_hook::set_once();
    colored::control::set_override(true);

    let mut output = Vec::new();
    let mut graph = Vec::new();
    compile("input", &source, &mut output, Some(&mut graph)).unwrap();

    vec![
        String::from_utf8(output).unwrap(),
        String::from_utf8(graph).unwrap(),
    ]
}

pub fn compile(
    path: &str,
    source: &str,
    mut w: impl Write,
    graph: Option<impl Write>,
) -> io::Result<()> {
    let source_file = match wipple_visualizer_syntax::SourceFile::parse(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            write!(w, "syntax error: {error}")?;
            return Ok(());
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

    let mut lowered = wipple_visualizer_lower::visit(&source_file, make_span);

    let definition_constraints = mem::take(&mut lowered.definition_constraints);
    let program_constraints = mem::take(&mut lowered.program_constraints);

    let mut provider = Provider {
        source,
        lowered: &mut lowered,
    };

    let mut ty_groups = {
        let mut typechecker = Typechecker::with_provider(&mut provider);
        typechecker.insert_constraints(definition_constraints);
        typechecker.insert_constraints(program_constraints);
        typechecker.to_ty_groups()
    };

    let nodes = provider
        .lowered
        .facts
        .iter()
        .filter(|(_, facts)| !facts.iter().any(Fact::is_hidden))
        .map(|(&node, _)| node)
        .collect::<Vec<_>>();

    let is_expression = |node: &NodeId| !provider.lowered.definitions.contains_key(node);

    for node in nodes.iter().copied().filter(is_expression) {
        // Give untyped expressions a default type of `Unknown`
        if ty_groups.index_of(node).is_none() {
            provider
                .lowered
                .facts
                .entry(node)
                .or_default()
                .push(Fact::new("unknownType", ()));

            let group_index = ty_groups.insert_group(Ty::Unknown);
            ty_groups.assign_node_to_index(node, group_index);
        }

        let tys = ty_groups
            .index_of(node)
            .map(|index| ty_groups.tys_at(index))
            .unwrap();

        let mut all_incomplete = true;
        for ty in tys {
            all_incomplete &= ty.is_incomplete();

            provider
                .lowered
                .facts
                .entry(node)
                .or_default()
                .push(Fact::new("type", ty.clone()));
        }

        if all_incomplete {
            provider
                .lowered
                .facts
                .entry(node)
                .or_default()
                .push(Fact::new("incompleteType", ()));
        }
    }

    display::write_tree(w, nodes, &provider)?;

    if let Some(graph) = graph {
        display::write_graph(graph, &ty_groups, &provider)?;
    }

    Ok(())
}

struct Provider<'a> {
    source: &'a str,
    lowered: &'a mut wipple_visualizer_lower::visitor::Result,
}

impl wipple_visualizer_typecheck::TypeProvider for Provider<'_> {
    fn copy_node(&mut self, node_id: NodeId) -> NodeId {
        let new_id = self.lowered.next_id;
        self.lowered.next_id.0 += 1;

        let span = self.lowered.spans.get(&node_id).unwrap().clone();
        self.lowered.spans.insert(new_id, span);

        new_id
    }

    fn get_trait_instances(
        &mut self,
        trait_id: NodeId,
    ) -> Vec<(NodeId, std::collections::BTreeMap<NodeId, NodeId>)> {
        self.lowered
            .instances
            .get(&trait_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .cloned()
            .map(|node| {
                let definition = self.lowered.definitions.get(&node).unwrap();

                let Definition::Instance(instance) = definition else {
                    unreachable!()
                };

                (node, instance.substitutions.clone())
            })
            .collect()
    }

    fn flag_resolved(
        &mut self,
        node: NodeId,
        bound: wipple_visualizer_typecheck::Bound,
        instance: NodeId,
    ) {
        let facts = self.lowered.facts.entry(node).or_default();
        facts.push(Fact::new("resolvedTrait", bound.tr));
        facts.push(Fact::new("resolvedTrait", instance));
    }

    fn flag_unresolved(&mut self, node: NodeId, bound: wipple_visualizer_typecheck::Bound) {
        self.lowered
            .facts
            .entry(node)
            .or_default()
            .push(Fact::new("unresolvedTrait", bound.tr));
    }
}

impl wipple_visualizer_typecheck::DisplayProvider for Provider<'_> {
    fn node_facts(&self, node: NodeId) -> &[Fact] {
        self.lowered
            .facts
            .get(&node)
            .map(|facts| facts.as_slice())
            .unwrap_or_default()
    }

    fn node_span_source(&self, node: NodeId) -> (Span, String) {
        let span = self.lowered.spans.get(&node).unwrap().clone();

        let mut source = self.source[span.range.clone()].to_string();

        // HACK: Remove comments
        source = source
            .lines()
            .skip_while(|line| line.is_empty() || line.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        (span, source)
    }

    fn node_comments(&self, node: NodeId) -> Option<String> {
        self.lowered
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
    }
}
