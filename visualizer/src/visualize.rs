use crate::{Db, Ty, TyGroups};
use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    hash::Hash,
    io::{self, Write},
};

pub trait WriteGraphContext<'a> {
    type Node: Debug + Copy + Eq + Ord + Hash;
    type Db: Db<Node = Self::Node>;

    fn format_node(&self, node: Self::Node) -> String;
    fn format_ty(&self, ty: &Ty<Self::Db>) -> String;

    fn include_node(&self, node: Self::Node) -> bool;
    fn related_nodes(&self, node: Self::Node) -> Vec<(Self::Node, String)>;

    fn node_span_source(&self, node: Self::Node) -> Option<(String, String)>;
    fn node_comments(&self, node: Self::Node) -> Option<String>;
}

pub fn write_graph<'a, Ctx: WriteGraphContext<'a>>(
    mut w: impl Write,
    ctx: Ctx,
    ty_groups: &TyGroups<Ctx::Db>,
    nodes: &[Ctx::Node],
) -> io::Result<()> {
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

        if !ctx.include_node(node) {
            continue;
        }

        let Some((node_span, node_source)) = ctx.node_span_source(node) else {
            continue;
        };

        for (related, relation) in ctx.related_nodes(node) {
            if !nodes.contains(&related) || !ctx.include_node(related) {
                continue;
            }

            if visited_relations
                .get(&(node, related))
                .is_some_and(|existing| existing.contains(&relation))
            {
                continue;
            }

            writeln!(
                w,
                "{}-- {} -->{}",
                ctx.format_node(node),
                relation,
                ctx.format_node(related)
            )?;

            visited_relations
                .entry((related, node))
                .or_default()
                .insert(relation);
        }

        let mut description = format!("{node_span:?}\n<pre>{node_source}</pre>");

        if let Some(comments) = ctx.node_comments(node) {
            description.push_str(&comments);
        }

        writeln!(
            w,
            "{}@{{ label: {:?} }}",
            ctx.format_node(node),
            description
        )?;
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
            .map(|ty| ctx.format_ty(ty))
            .collect::<Vec<_>>()
            .join(" or ");

        writeln!(w, "subgraph group{index}[\"<code>{description}</code>\"]")?;

        for node in nodes {
            if !ctx.include_node(node) {
                continue;
            }

            writeln!(w, "{}", ctx.format_node(node))?;
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
