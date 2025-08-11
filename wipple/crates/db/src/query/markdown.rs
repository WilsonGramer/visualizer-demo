use crate::{
    Db, FactValue, Span,
    query::{Query, Term, query},
};
use colored::Colorize;
use regex::Regex;
use std::{ops::Range, str::FromStr, sync::LazyLock};

pub trait MarkdownQueryExt<'a>: Sized {
    fn markdown(
        markdown: &str,
        matcher: impl Fn(&Db, &dyn FactValue, &str) -> bool + Send + Sync + 'a,
    ) -> Option<Self>;
}

impl<'a> MarkdownQueryExt<'a> for Query<'a, Vec<(Span, String)>> {
    fn markdown(
        markdown: &str,
        matcher: impl Fn(&Db, &dyn FactValue, &str) -> bool + Send + Sync + 'a,
    ) -> Option<Self> {
        let file = File::from_str(markdown).ok()?;

        Some(Query::new(move |db, initial| {
            let mut result = Vec::new();
            for values in query(&file.terms, initial, db, &matcher) {
                let mut body = file.body.clone();
                for link in file.links.iter().rev() {
                    if let Some(value) = values.get(&link.name).and_then(|value| value.display(db))
                    {
                        body.replace_range(
                            link.range.clone(),
                            &if link.code {
                                format!("`{value}`").blue().to_string()
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
                    result.push((span.clone(), body));
                }
            }

            result.sort_by_key(|(span, _)| (span.path.clone(), span.range.start, span.range.end));
            result.dedup();

            result
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct File {
    terms: Vec<Term>,
    body: String,
    links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq)]
struct Link {
    range: Range<usize>,
    name: String,
    code: bool,
}

static FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*---\n(?<frontmatter>(?s).*)\n---\n(?<body>(?s).*)$"#).unwrap()
});

static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\[(?<code>`)?(?<name>[A-Za-z_]+)`?\]"#).unwrap());

impl FromStr for File {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let file = FILE_REGEX
            .captures(s)
            .ok_or_else(|| anyhow::format_err!("missing frontmatter"))?;

        let frontmatter = file.name("frontmatter").unwrap().as_str();
        let body = file.name("body").unwrap().as_str().trim().to_string();

        let terms = frontmatter
            .lines()
            .map(|line| line.parse())
            .collect::<anyhow::Result<Vec<Term>>>()?;

        let links = LINK_REGEX
            .captures_iter(&body)
            .map(|captures| Link {
                range: captures.get(0).unwrap().range(),
                name: captures.name("name").unwrap().as_str().to_string(),
                code: captures.name("code").is_some(),
            })
            .collect::<Vec<_>>();

        Ok(File { terms, body, links })
    }
}
