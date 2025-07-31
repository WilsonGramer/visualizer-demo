use crate::{TyGroups, feedback::FeedbackProvider};
use itertools::Itertools;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::{self, Write},
};
use wipple_compiler_trace::{NodeId, Rule};

pub fn write_graph(
    w: &mut dyn Write,
    ty_groups: &TyGroups,
    rules: &BTreeMap<NodeId, HashSet<Rule>>,
    relations: &DiGraphMap<NodeId, Rule>,
    provider: &FeedbackProvider<'_>,
) -> fmt::Result {
    let node_id = |node: NodeId| format!("node{}", node.0);

    writeln!(w, "%%{{init: {{'theme':'neutral'}}}}%%")?;
    writeln!(w, "flowchart TD")?;
    writeln!(
        w,
        "classDef success fill:#0000ff10,stroke:#0000ff,stroke-dasharray:10;"
    )?;
    writeln!(
        w,
        "classDef error fill:#ff000010,stroke:#ff0000,stroke-dasharray:10;"
    )?;

    let mut visited_rules = BTreeMap::<_, BTreeSet<_>>::new();

    for node in ty_groups.nodes().chain(rules.keys().copied()) {
        if rules
            .get(&node)
            .is_some_and(|rules| rules.iter().any(Rule::should_ignore))
        {
            continue;
        }

        let (node_span, node_source) = provider.node_span_source(node);

        // Also link related nodes
        for parent in relations.neighbors_directed(node, Direction::Incoming) {
            if rules
                .get(&parent)
                .is_some_and(|rules| rules.iter().any(Rule::should_ignore))
            {
                continue;
            }

            let &rule = relations.edge_weight(parent, node).unwrap();

            if visited_rules
                .get(&(node, parent))
                .is_some_and(|existing| existing.contains(&rule))
            {
                continue;
            }

            writeln!(
                w,
                "{}-- {:?} -->{}",
                node_id(node),
                format!("{rule:?}"),
                node_id(parent)
            )?;

            visited_rules
                .entry((node, parent))
                .or_default()
                .insert(rule);
        }

        let rules = rules
            .get(&node)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|rule| rule.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let mut description = format!("{node_span:?}\n<pre>{node_source}</pre>\n{rules}");

        if let Some(comments) = provider.comments(node) {
            description.push_str(&comments);
        }

        writeln!(w, "{}@{{ label: {:?} }}", node_id(node), description)?;
    }

    let visited = visited_rules
        .into_keys()
        .flat_map(|(node, parent)| [node, parent])
        .collect::<BTreeSet<_>>();

    for (index, group_tys) in ty_groups.groups() {
        let nodes = ty_groups
            .nodes_in_group(index)
            .filter(|node| visited.contains(node))
            .collect::<Vec<_>>();

        if nodes.is_empty() {
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

        for node in nodes {
            writeln!(w, "{}", node_id(node))?;
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
