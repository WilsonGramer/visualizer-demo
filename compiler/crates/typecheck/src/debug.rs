use crate::{
    TyGroups,
    feedback::FeedbackProvider,
    util::{Fact, NodeId},
};
use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::{self, Write},
};

pub fn write_graph(
    w: &mut dyn Write,
    ty_groups: &TyGroups,
    facts: &BTreeMap<NodeId, HashSet<Fact>>,
    provider: &FeedbackProvider<'_>,
    filter: impl Fn(NodeId) -> bool,
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

    let mut visited_relations = BTreeMap::<_, BTreeSet<_>>::new();

    for node in ty_groups
        .nodes()
        .chain(facts.keys().copied())
        .filter(|&node| filter(node))
    {
        let Some(node_facts) = facts.get(&node) else {
            continue;
        };

        if node_facts.iter().any(Fact::should_ignore) {
            continue;
        }

        let (node_span, node_source) = provider.node_span_source(node);

        // Also link related nodes
        for (parent, relation) in node_facts.iter().filter_map(|fact| match fact {
            Fact::Node(parent, relation) => Some((*parent, relation.as_ref())),
            _ => None,
        }) {
            let Some(parent_facts) = facts.get(&parent) else {
                continue;
            };

            if parent_facts.iter().any(Fact::should_ignore) {
                continue;
            }

            if visited_relations
                .get(&(node, parent))
                .is_some_and(|existing| existing.contains(&relation))
            {
                continue;
            }

            writeln!(w, "{}-- {} -->{}", node_id(node), relation, node_id(parent))?;

            visited_relations
                .entry((node, parent))
                .or_default()
                .insert(relation);
        }

        let facts = node_facts
            .iter()
            .filter(|fact| !fact.should_ignore() && !matches!(fact, Fact::Node(_, _)))
            .map(|fact| fact.to_string())
            .collect::<Vec<_>>()
            .join(",\n");

        let mut description = format!("{node_span:?}\n<pre>{node_source}</pre>\n{facts}");

        if let Some(comments) = provider.node_comments(node) {
            description.push_str(&comments);
        }

        writeln!(w, "{}@{{ label: {:?} }}", node_id(node), description)?;
    }

    for (index, group_tys) in ty_groups.groups() {
        let nodes = ty_groups
            .nodes_in_group(index)
            .filter(|&node| filter(node))
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
            .join(" or ");

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
