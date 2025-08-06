use crate::{Db, Fact, NodeId};
use colored::Colorize;
use std::io::{self, Write};

impl Db {
    pub fn write(&self, mut w: impl Write, nodes: &[NodeId]) -> io::Result<()> {
        for &node in nodes {
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

        Ok(())
    }
}
