use colored::Colorize;
use std::io::{self, Write};
use wipple_visualizer_lower::fact_is_hidden;
use wipple_visualizer_typecheck::{DisplayProvider, NodeId};

pub fn write_tree(
    mut w: impl Write,
    nodes: &[NodeId],
    display: &dyn DisplayProvider,
) -> io::Result<()> {
    for &node in nodes {
        let facts = display.node_facts(node);
        if facts.is_empty() || facts.iter().any(fact_is_hidden) {
            continue;
        }

        let (_, source) = display.node_span_source(node);

        writeln!(w, "{}: {}", format!("{node:?}").bold(), source.blue())?;

        for fact in facts {
            write!(w, "  {}.{}", fact.namespace(), fact.name())?;

            let value = fact.value();

            if let Some(str) = value.display(display) {
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
