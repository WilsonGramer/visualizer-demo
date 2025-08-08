use line_index::LineIndex;
use regex::Regex;
use std::{fmt::Display, ops::Range, str::FromStr, sync::LazyLock};
use wipple_db::Span;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParsedSpan {
    LineCol {
        path: String,
        start: (usize, usize),
        end: (usize, usize),
    },
    Range {
        path: String,
        range: Range<usize>,
    },
}

impl FromStr for ParsedSpan {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static LINE_COL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"^(?<path>.*):(?<start_line>\d+):(?<start_col>\d+)-(?<end_line>\d+):(?<end_col>\d+)$"#).unwrap()
        });

        static RANGE_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"^(?<path>.*):(?<start>\d+)..(?<end>\d+)$"#).unwrap());

        if let Some(captures) = LINE_COL_REGEX.captures(s) {
            let path = captures.name("path").unwrap().as_str().to_string();

            let start_line = captures
                .name("start_line")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid start line number"))?;

            let start_col = captures
                .name("start_col")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid start column number"))?;

            let end_line = captures
                .name("end_line")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid end line number"))?;

            let end_col = captures
                .name("end_col")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid end column number"))?;

            Ok(ParsedSpan::LineCol {
                path,
                start: (start_line, start_col),
                end: (end_line, end_col),
            })
        } else if let Some(captures) = RANGE_REGEX.captures(s) {
            let path = captures.name("path").unwrap().as_str().to_string();

            let start = captures
                .name("line")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid start line number"))?;

            let end = captures
                .name("col")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .map_err(|_| anyhow::format_err!("invalid end column number"))?;

            Ok(ParsedSpan::Range {
                path,
                range: start..end,
            })
        } else {
            Err(anyhow::format_err!("invalid span: {s}"))
        }
    }
}

impl Display for ParsedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedSpan::LineCol {
                path,
                start: (start_line, start_col),
                end: (end_line, end_col),
            } => {
                write!(f, "{path}:{start_line}:{start_col}-{end_line}:{end_col}")
            }
            ParsedSpan::Range { path, range } => {
                write!(f, "{}:{}..{}", path, range.start, range.end)
            }
        }
    }
}

impl ParsedSpan {
    pub fn to_span(&self, line_index: &LineIndex) -> Option<Span> {
        Some(match *self {
            ParsedSpan::LineCol {
                ref path,
                start: (start_line, start_col),
                end: (end_line, end_col),
            } => Span {
                path: path.clone(),
                range: line_index
                    .offset(line_index::LineCol {
                        line: start_line as u32 - 1,
                        col: start_col as u32 - 1,
                    })?
                    .into()
                    ..line_index
                        .offset(line_index::LineCol {
                            line: end_line as u32 - 1,
                            col: end_col as u32 - 1,
                        })?
                        .into(),
                start_line_col: (start_line, start_col),
                end_line_col: (end_line, end_col),
            },
            ParsedSpan::Range {
                ref path,
                ref range,
            } => {
                let start_line_col = line_index.try_line_col((range.start as u32).into())?;
                let end_line_col = line_index.try_line_col((range.end as u32).into())?;

                Span {
                    path: path.clone(),
                    range: range.clone(),
                    start_line_col: (
                        start_line_col.line as usize + 1,
                        start_line_col.col as usize + 1,
                    ),
                    end_line_col: (
                        end_line_col.line as usize + 1,
                        end_line_col.col as usize + 1,
                    ),
                }
            }
        })
    }
}
