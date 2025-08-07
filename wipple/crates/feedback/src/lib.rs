mod parser;
mod query;

pub use parser::File;

use itertools::Itertools;
use wipple_db::{Db, Span};

pub fn iter_feedback<'a>(
    db: &Db,
    files: impl IntoIterator<Item = &'a File>,
) -> impl Iterator<Item = (Span, String)> {
    let mut feedback = Vec::new();
    for file in files {
        for values in file.query(db) {
            let mut body = file.body.clone();
            for link in file.links.iter().rev() {
                if let Some(value) = values.get(&link.name).and_then(|value| value.display(db)) {
                    body.replace_range(
                        link.range.clone(),
                        &if link.code {
                            format!("`{value}`")
                        } else {
                            value
                        },
                    );
                }
            }

            if let Some(span) = values
                .get("span")
                .and_then(|value| value.downcast_ref::<Span>())
            {
                feedback.push((span.clone(), body));
            }
        }
    }

    feedback.into_iter().unique()
}
