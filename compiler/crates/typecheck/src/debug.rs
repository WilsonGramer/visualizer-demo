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
    let node_id = |node: NodeId| format!("node{}", node.0);

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

        let (node_span, node_source) = provider.node_span_source(node);

        // Also link related nodes
        for parent in relations.neighbors_directed(node, Direction::Incoming) {
            if !filter(parent) {
                continue;
            }

            let &rule = relations.edge_weight(parent, node).unwrap();

            writeln!(
                w,
                "{}-- {:?} -->{}",
                node_id(node),
                format!("{rule:?}"),
                node_id(parent)
            )?;

            visited.insert(parent);
        }

        writeln!(
            w,
            "{}@{{ label: {:?} }}",
            node_id(node),
            format!("{node_span:?}\n<pre>{node_source}</pre>")
        )?;

        visited.insert(node);
    }

    for (index, group_tys) in ty_groups.groups() {
        let error = !group_tys.iter().all_equal();

        let description = group_tys
            .iter()
            .unique()
            .map(|ty| ty.to_debug_string(provider))
            .collect::<Vec<_>>()
            .join("\n");

        writeln!(w, "subgraph group{index}[\"<code>{description}</code>\"]")?;

        for node in ty_groups.nodes_in_group(index) {
            if visited.contains(&node) {
                writeln!(w, "{}", node_id(node))?;
            }
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
