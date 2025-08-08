use crate::matcher::matcher;
use colored::Colorize;
use std::io::{self, Write};
use wipple_db::{Db, MarkdownQueryExt, Query};

#[derive(rust_embed::RustEmbed)]
#[folder = "feedback"]
struct Feedback;

pub fn write_feedback(db: &Db, mut output: impl Write) -> io::Result<()> {
    let queries = Feedback::iter()
        .filter(|path| path.ends_with(".md"))
        .map(|path| {
            let markdown = String::from_utf8(Feedback::get(&path).unwrap().data.to_vec()).unwrap();

            Query::markdown(&markdown, matcher)
                .unwrap_or_else(|| panic!("invalid feedback file: {path}"))
        });

    for (span, message) in queries.flat_map(|query| query.run(db, Default::default())) {
        let message = textwrap::wrap(
            &message,
            textwrap::Options::new(80)
                .initial_indent("    ")
                .subsequent_indent("    "),
        )
        .join("\n");

        writeln!(
            output,
            "{}\n\n{}\n",
            format!("Feedback on {span:?}:").bold().underline(),
            message
        )?;
    }

    Ok(())
}
