use crate::{
    Db, FactValue, Span,
    query::{Arg, Query, Term, query},
};
use regex::Regex;
use std::{ops::Range, str::FromStr, sync::LazyLock};

pub trait MarkdownQueryExt<'a>: Sized {
    fn markdown(
        markdown: &str,
        matcher: impl Fn(&Db, &dyn FactValue, &str) -> bool + 'a,
    ) -> Option<Self>;
}

impl<'a> MarkdownQueryExt<'a> for Query<'a, (Span, String)> {
    fn markdown(
        markdown: &str,
        matcher: impl Fn(&Db, &dyn FactValue, &str) -> bool + 'a,
    ) -> Option<Self> {
        let file = File::from_str(markdown).ok()?;

        Some(Query::new(move |db| {
            let mut result = Vec::new();
            for values in query(&file.terms, db, &matcher) {
                let mut body = file.body.clone();
                for link in file.links.iter().rev() {
                    if let Some(value) = values.get(&link.name).and_then(|value| value.display(db))
                    {
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
                    result.push((span.clone(), body));
                }
            }

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

static TERM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(?<node>[A-Za-z_]+)\.(?<fact>[A-Za-z_]+)(\((?<value>`[^`]*`|[A-Za-z_]+)\))?$"#)
        .unwrap()
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
            .map(|line| {
                let captures = TERM_REGEX
                    .captures(line)
                    .ok_or_else(|| anyhow::format_err!("invalid term: {line}"))?;

                Ok(Term {
                    node: captures.name("node").unwrap().as_str().to_string(),
                    fact: captures.name("fact").unwrap().as_str().to_string(),
                    arg: captures.name("value").map(|c| {
                        if c.as_str().starts_with("`") {
                            Arg::Value(c.as_str()[1..(c.len() - 1)].to_string())
                        } else {
                            Arg::Variable(c.as_str().to_string())
                        }
                    }),
                })
            })
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
