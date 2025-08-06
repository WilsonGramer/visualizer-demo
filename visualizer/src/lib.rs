pub mod graph;

pub use visualizer_db as db;
pub use visualizer_typecheck as typecheck;

use crate::graph::write_graph;
use std::io::{self, Write};
use visualizer_db::{Db, Span};
use visualizer_typecheck::TyGroups;

#[derive(Debug, Clone, Copy)]
pub enum Filter<'a> {
    Range(u32, u32),
    Lines(&'a [u32]),
}

pub fn visualize(
    db: &Db,
    ty_groups: &TyGroups,
    filter: Option<Filter<'_>>,
    w: impl Write,
    graph: Option<impl Write>,
) -> io::Result<()> {
    let nodes = db
        .nodes()
        .filter(|&node| !db.is_hidden(node))
        .filter(|&node| {
            let Some(filter) = filter else {
                return true;
            };

            let Some(span) = db.get::<Span>(node, "span") else {
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

    db.write(w, &nodes)?;

    if let Some(graph) = graph {
        write_graph(graph, db, ty_groups, &nodes)?;
    }

    Ok(())
}
