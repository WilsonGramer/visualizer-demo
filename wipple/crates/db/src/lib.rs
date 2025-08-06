mod db;
mod fact;
mod node;
mod span;

pub use db::*;
pub use fact::*;
pub use node::*;
pub use span::*;

use colored::Colorize;
use std::io::{self, Write};
use visualizer::TyGroups;

#[derive(Debug, Clone, Copy)]
pub enum Filter<'a> {
    Range(u32, u32),
    Lines(&'a [u32]),
}

impl Db {
    pub fn write(
        &self,
        ty_groups: &TyGroups<Db>,
        filter: Option<Filter<'_>>,
        mut w: impl Write,
        graph: Option<impl Write>,
    ) -> io::Result<()> {
        let nodes = self
            .nodes()
            .filter(|&node| !self.is_hidden(node))
            .filter(|&node| {
                let Some(filter) = filter else {
                    return true;
                };

                let Some(span) = self.get::<Span>(node, "span") else {
                    return false;
                };

                match filter {
                    Filter::Range(start, end) => {
                        span.range.start <= (end as usize) && span.range.end >= (start as usize)
                    }
                    Filter::Lines(lines) => lines.contains(&(span.start_line_col.0 as u32)),
                }
            })
            .collect::<Vec<_>>();

        for &node in &nodes {
            let facts = self.iter(node).collect::<Vec<_>>();

            if facts.iter().copied().any(Fact::is_hidden) {
                continue;
            }

            let Some(source) = self.get::<String>(node, "source") else {
                continue;
            };

            writeln!(w, "{}: {}", format!("{node:?}").bold(), source.blue())?;

            for fact in facts {
                write!(w, "  {}", fact.name())?;

                let value = fact.value();

                if let Some(str) = value.display(self) {
                    if value.is_code() {
                        writeln!(w, "({})", str.blue())?;
                    } else {
                        writeln!(w, "({str})")?;
                    }
                } else {
                    writeln!(w)?;
                }
            }
        }

        if let Some(mut graph) = graph {
            visualizer::write_graph(&mut graph, Ctx(self), ty_groups, &nodes)?;
        }

        Ok(())
    }
}

struct Ctx<'a>(&'a Db);

impl<'a> visualizer::WriteGraphContext<'a> for Ctx<'a> {
    type Node = NodeId;
    type Db = Db;

    fn format_node(&self, node: Self::Node) -> String {
        format!("node{}", node.0)
    }

    fn format_ty(&self, ty: &visualizer::Ty<Self::Db>) -> String {
        ty.display(self.0).unwrap()
    }

    fn include_node(&self, node: Self::Node) -> bool {
        !self.0.is_hidden(node)
    }

    fn related_nodes(&self, node: Self::Node) -> Vec<(Self::Node, String)> {
        self.0
            .iter(node)
            .filter_map(|fact| {
                Some((
                    *fact.value().downcast_ref::<NodeId>()?,
                    fact.name().to_string(),
                ))
            })
            .collect()
    }

    fn node_span_source(&self, node: Self::Node) -> Option<(String, String)> {
        let span = self.0.get::<Span>(node, "span")?;

        let source = self.0.get::<String>(node, "source")?;

        // Remove comments
        let source = source
            .lines()
            .skip_while(|line| line.is_empty() || line.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        Some((format!("{span:?}"), source.clone()))
    }

    fn node_comments(&self, node: Self::Node) -> Option<String> {
        self.0.get::<String>(node, "comments").cloned()
    }
}
