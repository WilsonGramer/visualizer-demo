use crate::{
    Db, Source, Span,
    fact::{Fact, FactValue},
    node::NodeId,
};
use colored::Colorize;
use std::io::{self, Write};
use visualizer::{Graph, TyGroups};

#[derive(Debug, Clone, Copy)]
pub enum Filter<'a> {
    Range(u32, u32),
    Lines(&'a [u32]),
}

impl Db {
    pub fn write(&self, filter: &[Filter<'_>], indent: &str, mut w: impl Write) -> io::Result<()> {
        let nodes = self.filtered_nodes(filter).collect::<Vec<_>>();

        for &node in &nodes {
            let mut facts = self.iter(node).collect::<Vec<_>>();

            if facts.iter().copied().any(Fact::is_hidden) {
                continue;
            }

            let Some(Source(source)) = self.get::<Source>(node, "source") else {
                continue;
            };

            writeln!(
                w,
                "{indent}{}: {}",
                format!("{node:?}").bold(),
                source.blue()
            )?;

            facts.sort_by_key(|fact| fact.name());

            for fact in facts {
                write!(w, "{indent}{indent}{}", fact.name())?;

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

        Ok(())
    }

    pub fn graph(&self, ty_groups: &TyGroups<Db>, filter: &[Filter<'_>]) -> Graph {
        Graph::generate(Ctx(self), ty_groups, self.filtered_nodes(filter))
    }

    fn filtered_nodes(&self, filter: &[Filter<'_>]) -> impl Iterator<Item = NodeId> {
        self.nodes()
            .filter(|&node| !self.is_hidden(node))
            .filter(move |&node| {
                if filter.is_empty() {
                    return true;
                }

                let Some(span) = self.get::<Span>(node, "span") else {
                    return false;
                };

                filter.iter().any(|&filter| match filter {
                    Filter::Range(start, end) => {
                        span.range.start >= (start as usize) && span.range.end <= (end as usize)
                    }
                    Filter::Lines(lines) => lines.contains(&(span.start_line_col.0 as u32)),
                })
            })
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

    fn node_data(&self, node: Self::Node) -> serde_json::Value {
        let span = self
            .0
            .get::<Span>(node, "span")
            .map(|span| span.display(self.0));

        let source = self.0.get::<Source>(node, "source").map(|Source(source)| {
            // Remove comments
            source
                .lines()
                .skip_while(|line| line.is_empty() || line.starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
        });

        let comments = self
            .0
            .get::<Source>(node, "comments")
            .map(|Source(source)| source);

        serde_json::json!({
            "span": span,
            "source": source,
            "comments": comments,
        })
    }
}
