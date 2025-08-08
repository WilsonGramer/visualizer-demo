mod markdown;
mod yaml;

pub use markdown::*;
pub use yaml::*;

use crate::{Db, FactValue, NodeId};
use regex::Regex;
use std::{borrow::Cow, collections::HashMap, rc::Rc, str::FromStr, sync::LazyLock};

pub type QueryValues = HashMap<String, Rc<dyn FactValue>>;

pub struct Query<'a, T>(Box<dyn Fn(&Db, QueryValues) -> T + Send + Sync + 'a>);

impl<'a, T> Query<'a, T> {
    pub fn new(f: impl Fn(&Db, QueryValues) -> T + Send + Sync + 'a) -> Self {
        Query(Box::new(f))
    }

    pub fn run(&self, db: &Db, initial: QueryValues) -> T {
        (self.0)(db, initial)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub not: bool,
    pub node: String,
    pub fact: String,
    pub arg: Option<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    Variable(String),
    Value(String),
}

pub fn query(
    terms: &[Term],
    initial: QueryValues,
    db: &Db,
    matcher: impl Fn(&Db, &dyn FactValue, &str) -> bool,
) -> impl Iterator<Item = QueryValues> {
    let mut result = Vec::new();
    query_inner(db, &matcher, terms, &initial, &mut result);
    result.into_iter()
}

fn query_inner(
    db: &Db,
    matcher: &dyn Fn(&Db, &dyn FactValue, &str) -> bool,
    terms: &[Term],
    values: &QueryValues,
    result: &mut Vec<QueryValues>,
) {
    match terms.split_first() {
        Some((next, terms)) => {
            let facts: Vec<_> = match values
                .get(&next.node)
                .and_then(|node| node.downcast_ref::<NodeId>().copied())
                .filter(|&node| !db.is_hidden(node))
            {
                Some(node) => db
                    .iter_by(node, &next.fact)
                    .map(|fact| (Cow::Borrowed(values), fact))
                    .collect(),
                None => db
                    .all(&next.fact)
                    .map(|(node, fact)| {
                        let mut values = values.clone();
                        values.insert(next.node.clone(), Rc::new(node));
                        (Cow::Owned(values), fact)
                    })
                    .collect(),
            };

            if next.not {
                if facts.is_empty() {
                    query_inner(db, matcher, terms, values, result);
                } else {
                    return;
                }
            }

            for (mut values, fact) in facts {
                if let Some(arg) = &next.arg {
                    match arg {
                        Arg::Variable(variable) => match values.get(variable) {
                            Some(other) => {
                                if other.as_ref() != fact.value() {
                                    continue;
                                }
                            }
                            None => {
                                values.to_mut().insert(variable.clone(), fact.clone_value());
                            }
                        },
                        Arg::Value(pattern) => {
                            if !matcher(db, fact.value(), pattern) {
                                continue;
                            }
                        }
                    }
                }

                query_inner(db, matcher, terms, &values, result);
            }
        }
        None => result.push(values.clone()),
    }
}

impl FromStr for Term {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static TERM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r#"^(?<not>!)?(?<node>[A-Za-z_]+)\.(?<fact>[A-Za-z_]+)(\((?<value>`[^`]*`|[A-Za-z_]+)\))?$"#,
            )
            .unwrap()
        });

        let captures = TERM_REGEX
            .captures(s)
            .ok_or_else(|| anyhow::format_err!("invalid term: {s}"))?;

        Ok(Term {
            not: captures.name("not").is_some(),
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
    }
}
