use crate::{context::FeedbackProvider, session::Session};
use itertools::Itertools;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};

impl Session<'_> {
    pub fn write_debug_graph(
        &self,
        w: &mut dyn Write,
        relations: &DiGraphMap<NodeId, Rule>,
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

        let mut exclude = BTreeSet::new();

        for &node in self.constraints.tys.keys() {
            if !(self.filter)(node) {
                continue;
            }

            let (node_span, node_source) = provider.node_span_source(node);

            // Also link related nodes
            for parent in relations.neighbors_directed(node, Direction::Incoming) {
                let &rule = relations.edge_weight(parent, node).unwrap();

                if !rule.is(RuleCategory::Expression) {
                    exclude.insert(parent);
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

            if !exclude.contains(&node) {
                writeln!(
                    w,
                    "{}@{{ label: {:?} }}",
                    node_id(node),
                    format!("{node_span:?}\n{node_source}")
                )?;
            }
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
                .filter(|(node, _)| (self.filter)(*node))
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

            writeln!(w, "subgraph group{id}[\"**{description}**\"]")?;

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
