use crate::TyGroups;
use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
use visualizer_db::{Db, Fact, FactValue, NodeId, Span};

pub fn write_graph(
    mut w: impl Write,
    db: &Db,
    ty_groups: &TyGroups,
    nodes: &[NodeId],
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

    // Also show nodes that are in the same group as any node in `nodes`
    let mut visited_groups = BTreeSet::new();
    let mut visited_nodes = BTreeSet::new();
    for &node in nodes {
        visited_nodes.insert(node);

        if let Some(group_index) = ty_groups.index_of(node) {
            visited_groups.insert(group_index);
            visited_nodes.extend(ty_groups.nodes_in_group(group_index));
        }
    }

    let mut visited_relations = BTreeMap::<_, BTreeSet<_>>::new();
    for node in ty_groups.nodes() {
        if !visited_nodes.contains(&node) {
            continue;
        }

        let node_facts = db.iter(node).collect::<Vec<_>>();
        if !filter_facts(node_facts.iter().copied()) {
            continue;
        }

        let Some(node_span) = db.get::<Span>(node, "span") else {
            continue;
        };

        let Some(node_source) = db.get::<String>(node, "source") else {
            continue;
        };

        for parent in node_facts
            .iter()
            .filter_map(|fact| fact.value().downcast_ref::<NodeId>().copied())
        {
            if !nodes.contains(&parent) {
                continue;
            }

            let Some(relation) = get_relation(node_facts.iter().copied(), parent) else {
                continue;
            };

            let parent_facts = db.iter(parent).collect::<Vec<_>>();
            if !filter_facts(parent_facts.iter().copied()) {
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

        if let Some(comments) = db.get::<String>(node, "comments") {
            description.push_str(comments);
        }

        writeln!(w, "{}@{{ label: {:?} }}", node_id(node), description)?;
    }

    for (index, group_tys) in ty_groups.groups() {
        if !visited_groups.contains(&index) {
            continue;
        }

        let nodes = ty_groups.nodes_in_group(index).collect::<Vec<_>>();

        if nodes.is_empty() {
            continue;
        }

        let error = !group_tys.iter().all_equal();

        let description = group_tys
            .iter()
            .unique()
            .map(|ty| ty.display(db).unwrap())
            .collect::<Vec<_>>()
            .join(" or ");

        writeln!(w, "subgraph group{index}[\"<code>{description}</code>\"]")?;

        for node in nodes {
            if !filter_facts(db.iter(node)) {
                continue;
            }

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

fn filter_facts<'a>(facts: impl IntoIterator<Item = &'a Fact>) -> bool {
    !facts.into_iter().any(Fact::is_hidden)
}

fn get_relation<'a>(facts: impl IntoIterator<Item = &'a Fact>, parent: NodeId) -> Option<&'a str> {
    facts
        .into_iter()
        .find(|fact| {
            fact.value()
                .downcast_ref::<NodeId>()
                .is_some_and(|&node| node == parent)
        })
        .map(|fact| fact.name())
}
