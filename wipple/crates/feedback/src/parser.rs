use regex::Regex;
use std::{ops::Range, str::FromStr, sync::LazyLock};

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub terms: Vec<Term>,
    pub body: String,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub node: String,
    pub fact: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub range: Range<usize>,
    pub name: String,
    pub code: bool,
}

static FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*---\n(?<frontmatter>(?s).*)\n---\n(?<body>(?s).*)$"#).unwrap()
});

static TERM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(?<node>[A-Za-z_]+)\.(?<fact>[A-Za-z_]+)(\((?<value>[A-Za-z_]+)\))?$"#).unwrap()
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
                    key: captures.name("value").map(|c| c.as_str().to_string()),
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
