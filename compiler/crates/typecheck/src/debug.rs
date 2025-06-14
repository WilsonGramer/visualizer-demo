use crate::{
    constraints::Ty,
    context::FeedbackProvider,
    session::{Session, TyGroups},
};
use itertools::Itertools;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
use wipple_compiler_trace::{AnyRule, NodeId, Rule};

impl Session {
    pub fn write_debug_graph(
        &self,
        w: &mut dyn Write,
        groups: &TyGroups,
        tys: &BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
        relations: &DiGraphMap<NodeId, AnyRule>,
        provider: &FeedbackProvider<'_>,
    ) -> io::Result<()> {
        let node_id = |node: NodeId| format!("node{}", node.0);

        writeln!(
            w,
            "%%{{init: {{'layout':'elk','theme':'neutral','fontFamily':'Fira Code'}}}}%%"
        )?;
        writeln!(w, "flowchart LR")?;
        writeln!(
            w,
            "classDef success fill:#0000ff10,stroke:#0000ff,stroke-dasharray:10;"
        )?;
        writeln!(
            w,
            "classDef error fill:#ff000010,stroke:#ff0000,stroke-dasharray:10;"
        )?;

        for node in groups
            .0
            .borrow()
            .values()
            .flat_map(|group| group.borrow().iter().copied().collect::<Vec<_>>())
            .chain(self.nodes.iter().copied())
            .unique()
        {
            let (node_span, node_source) = provider.node_span_source(node);

            // Also link related nodes
            for parent in relations.neighbors_directed(node, Direction::Incoming) {
                let &rule = relations.edge_weight(parent, node).unwrap();

                if rule.kind().is_hidden() {
                    continue;
                }

                writeln!(
                    w,
                    "{}-- {:?} -->{}",
                    node_id(node),
                    format!("{rule:?}"),
                    node_id(parent)
                )?;
            }

            writeln!(
                w,
                "{}@{{ label: {:?} }}",
                node_id(node),
                format!("{node_span:?}\n{node_source}")
            )?;
        }

        let mut visited = BTreeSet::new();

        for (&id, group) in groups.0.borrow().iter() {
            let group_tys = group
                .borrow()
                .iter()
                .filter(|&node| !visited.contains(node))
                .flat_map(|node| tys.get(node).unwrap())
                .map(|(ty, _)| ty)
                .unique()
                .collect::<Vec<_>>();

            if group_tys.is_empty() {
                continue;
            }

            let error = group_tys.len() > 1; // mutiple possible types

            let group_tys = group_tys
                .iter()
                .map(|ty| ty.to_debug_string(provider))
                .collect::<Vec<_>>()
                .join("\n");

            writeln!(w, "subgraph group{id}[\"**{group_tys}**\"]")?;

            for &node in group.borrow().iter() {
                if !visited.contains(&node) {
                    visited.insert(node);
                    writeln!(w, "{}", node_id(node))?;
                }
            }

            writeln!(w, "end")?;

            writeln!(
                w,
                "class group{id} {}",
                if error { "error" } else { "success" }
            )?;
        }

        Ok(())
    }
}
