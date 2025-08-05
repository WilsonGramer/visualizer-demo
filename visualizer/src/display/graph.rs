use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
use wipple_visualizer_typecheck::{DisplayProvider, Fact, NodeId, TyGroups};

pub fn write_graph(
    mut w: impl Write,
    nodes: &[NodeId],
    ty_groups: &TyGroups,
    display: &dyn DisplayProvider,
) -> io::Result<()> {
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

    for &node in nodes {
        let node_facts = display.node_facts(node);
        if !filter_facts(node_facts) {
            continue;
        }

        let (node_span, node_source) = display.node_span_source(node);

        for parent in node_facts
            .iter()
            .filter_map(|fact| fact.value().downcast_ref::<NodeId>().copied())
        {
            if !nodes.contains(&parent) {
                continue;
            }

            let Some(relation) = get_relation(node_facts, parent) else {
                continue;
            };

            let parent_facts = display.node_facts(parent);
            if !filter_facts(parent_facts) {
                continue;
            }

            if visited_relations
                .get(&(node, parent))
                .is_some_and(|existing| existing.contains(relation))
            {
                continue;
            }

            writeln!(w, "{}-- {} -->{}", node_id(node), relation, node_id(parent))?;

            visited_relations
                .entry((parent, node))
                .or_default()
                .insert(relation);
        }

        let mut description = format!("{node_span:?}\n<pre>{node_source}</pre>");

        if let Some(comments) = display.node_comments(node) {
            description.push_str(&comments);
        }

        writeln!(w, "{}@{{ label: {:?} }}", node_id(node), description)?;
    }

    for (index, group_tys) in ty_groups.groups() {
        let nodes = ty_groups
            .nodes_in_group(index)
            .filter(|&node| nodes.contains(&node) && filter_facts(display.node_facts(node)))
            .collect::<Vec<_>>();

        if nodes.is_empty() {
            continue;
        }

        let error = !group_tys.iter().all_equal();

        let description = group_tys
            .iter()
            .unique()
            .map(|ty| ty.display(display))
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

fn filter_facts(facts: &[Fact]) -> bool {
    !facts.is_empty() && !facts.iter().any(Fact::is_hidden)
}

fn get_relation(facts: &[Fact], parent: NodeId) -> Option<&str> {
    facts
        .iter()
        .find(|fact| {
            fact.value()
                .downcast_ref::<NodeId>()
                .is_some_and(|&node| node == parent)
        })
        .map(|fact| fact.name())
}
