use crate::{context::FeedbackProvider, typechecker::TyGroups};
use itertools::Itertools;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};
use wipple_compiler_trace::{NodeId, Rule};

pub fn write_graph(
    w: &mut dyn Write,
    ty_groups: &TyGroups,
    relations: &DiGraphMap<NodeId, Rule>,
    provider: &FeedbackProvider<'_>,
    filter: impl Fn(NodeId) -> bool,
) -> io::Result<()> {
    let node_id = |node: NodeId| format!("node{}", node.index);

    writeln!(w, "%%{{init: {{'theme':'neutral'}}}}%%")?;
    writeln!(w, "flowchart LR")?;
    writeln!(
        w,
        "classDef success fill:#0000ff10,stroke:#0000ff,stroke-dasharray:10;"
    )?;
    writeln!(
        w,
        "classDef error fill:#ff000010,stroke:#ff0000,stroke-dasharray:10;"
    )?;

    let mut visited = BTreeSet::new();

    for node in ty_groups.nodes() {
        if !(filter)(node) {
            continue;
        }

        let display_node = provider.replacement_node(node).unwrap_or(node);
        let (node_span, node_source) = provider.node_span_source(display_node);

        // Also link related nodes
        for parent in relations.neighbors_directed(node, Direction::Incoming) {
            if !filter(parent) {
                continue;
            }

            let display_parent = provider.replacement_node(parent).unwrap_or(parent);

            let &rule = relations.edge_weight(parent, node).unwrap();

            writeln!(
                w,
                "{}-- {:?} -->{}",
                node_id(display_node),
                format!("{rule:?}"),
                node_id(display_parent)
            )?;

            visited.insert(display_parent);
        }

        let mut description = format!("{node_span:?}\n<pre>{node_source}</pre>");

        if let Some(comments) = provider.comments(display_node) {
            description.push_str(&comments);
        }

        writeln!(
            w,
            "{}@{{ label: {:?} }}",
            node_id(display_node),
            description
        )?;

        visited.insert(display_node);
    }

    for (index, group_tys) in ty_groups.groups() {
        let display_nodes = ty_groups
            .nodes_in_group(index)
            .map(|node| provider.replacement_node(node).unwrap_or(node))
            .filter(|display_node| visited.contains(display_node))
            .collect::<Vec<_>>();

        if display_nodes.is_empty() {
            continue;
        }

        let error = !group_tys.iter().all_equal();

        let description = group_tys
            .iter()
            .unique()
            .map(|ty| ty.to_debug_string(provider))
            .collect::<Vec<_>>()
            .join("\n");

        writeln!(w, "subgraph group{index}[\"<code>{description}</code>\"]")?;

        for display_node in display_nodes {
            writeln!(w, "{}", node_id(display_node))?;
        }

        writeln!(w, "end")?;

        writeln!(
            w,
            "class group{index} {}",
            if error { "error" } else { "success" }
        )?;
    }

    Ok(())
}
