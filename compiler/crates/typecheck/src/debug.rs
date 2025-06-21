use crate::{context::FeedbackProvider, typechecker::Typechecker};
use itertools::Itertools;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};
use wipple_compiler_trace::{NodeId, Rule};

impl Typechecker<'_> {
    pub fn write_debug_graph(
        &self,
        w: &mut dyn Write,
        definitions: &BTreeSet<NodeId>,
        relations: &DiGraphMap<NodeId, Rule>,
        provider: &FeedbackProvider<'_>,
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

        for &node in self.constraints.tys.keys().chain(definitions) {
            if !(self.filter)(node) && !definitions.contains(&node) {
                continue;
            }

            let (node_span, node_source) = provider.node_span_source(node);

            // Also link related nodes
            for parent in relations.neighbors_directed(node, Direction::Incoming) {
                if !(self.filter)(parent) && !definitions.contains(&parent) {
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

        let groups = self
            .constraints
            .tys
            .iter()
            .flat_map(|(&node, tys)| tys.iter().map(move |(ty, group)| (node, ty, *group)))
            .filter_map(|(node, ty, group)| Some((group?, (node, ty))))
            .into_group_map();

        for (id, group_tys) in groups {
            let group_tys = group_tys
                .into_iter()
                .filter(|&(node, _)| visited.contains(&node) && !definitions.contains(&node))
                .collect::<Vec<_>>();

            if group_tys.is_empty() {
                continue;
            }

            let error = !group_tys.iter().map(|&(_, ty)| ty).all_equal();

            let description = group_tys
                .iter()
                .map(|(_, ty)| ty)
                .unique()
                .map(|ty| ty.to_debug_string(provider))
                .collect::<Vec<_>>()
                .join("\n");

            writeln!(w, "subgraph group{id}[\"<code>{description}</code>\"]")?;

            for (node, _) in group_tys {
                writeln!(w, "{}", node_id(node))?;
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
