mod markdown;
pub use markdown::*;

use crate::{Db, FactValue, NodeId};
use std::{borrow::Cow, collections::HashMap};

pub struct Query<'a, T>(Box<dyn Fn(&Db) -> Vec<T> + 'a>);

impl<'a, T> Query<'a, T> {
    pub fn new(f: impl Fn(&Db) -> Vec<T> + 'a) -> Self {
        Query(Box::new(f))
    }

    pub fn run(&self, db: &Db) -> Vec<T> {
        (self.0)(db)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub node: String,
    pub fact: String,
    pub arg: Option<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    Variable(String),
    Value(String),
}

pub fn query<'a>(
    terms: &[Term],
    db: &'a Db,
    matcher: impl Fn(&'a Db, &dyn FactValue, &str) -> bool,
) -> impl Iterator<Item = HashMap<String, &'a dyn FactValue>> + 'a {
    let mut result = Vec::new();
    query_inner(
        db,
        &matcher,
        terms,
        &HashMap::new(),
        &HashMap::new(),
        &mut result,
    );

    result.into_iter()
}

fn query_inner<'a>(
    db: &'a Db,
    matcher: &dyn Fn(&'a Db, &dyn FactValue, &str) -> bool,
    terms: &[Term],
    nodes: &HashMap<String, NodeId>,
    values: &HashMap<String, &'a dyn FactValue>,
    result: &mut Vec<HashMap<String, &'a dyn FactValue>>,
) {
    match terms.split_first() {
        Some((next, terms)) => {
            let facts: Vec<_> = match nodes.get(&next.node).copied() {
                Some(node) => db
                    .iter_by(node, &next.fact)
                    .map(|fact| (Cow::Borrowed(nodes), fact))
                    .collect(),
                None => db
                    .all(&next.fact)
                    .map(|(node, fact)| {
                        let mut nodes = nodes.clone();
                        nodes.insert(next.node.clone(), node);
                        (Cow::Owned(nodes), fact)
                    })
                    .collect(),
            };

            for (nodes, fact) in facts {
                let values = if let Some(arg) = &next.arg {
                    let value = fact.value();

                    match arg {
                        Arg::Variable(variable) => {
                            if values.get(variable).is_some() {
                                continue;
                            }

                            let mut values = values.clone();
                            values.insert(variable.clone(), value);
                            Cow::Owned(values)
                        }
                        Arg::Value(pattern) => {
                            if !matcher(db, value, pattern) {
                                continue;
                            }

                            Cow::Borrowed(values)
                        }
                    }
                } else {
                    Cow::Borrowed(values)
                };

                query_inner(db, matcher, terms, &nodes, &values, result);
            }
        }
        None => result.push(values.clone()),
    }
}
