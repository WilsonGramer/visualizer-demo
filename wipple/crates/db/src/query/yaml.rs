use crate::{
    Db, FactValue, Query,
    query::{QueryValues, Term, query},
};
use saphyr::{LoadableYamlNode, Yaml};
use std::{collections::HashMap, str::FromStr};

pub trait YamlQueryExt<'a>: Sized {
    fn yaml(
        yaml: &str,
        matcher: &'a (dyn Fn(&Db, &dyn FactValue, &str) -> bool + Send + Sync),
    ) -> Option<HashMap<String, Self>>;
}

impl<'a> YamlQueryExt<'a> for Query<'a, Vec<QueryValues>> {
    fn yaml(
        yaml: &str,
        matcher: &'a (dyn Fn(&Db, &dyn FactValue, &str) -> bool + Send + Sync),
    ) -> Option<HashMap<String, Self>> {
        let file = File::from_str(yaml).ok()?;

        Some(
            file.queries
                .into_iter()
                .map(move |(action, options)| {
                    let query = Query::new(move |db, initial| {
                        let mut result = Vec::new();
                        for terms in &options {
                            result.extend(query(terms, initial.clone(), db, matcher));
                        }

                        result
                    });

                    (action, query)
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
struct File {
    queries: HashMap<String, Vec<Vec<Term>>>,
}

impl FromStr for File {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(File {
            queries: Yaml::load_from_str(s)?[0]
                .as_mapping()
                .ok_or_else(|| anyhow::format_err!("expected a mapping"))?
                .into_iter()
                .map(|(key, value)| {
                    Ok((
                        key.as_str()
                            .ok_or_else(|| anyhow::format_err!("expected key to be a string"))?
                            .to_string(),
                        value
                            .as_sequence()
                            .ok_or_else(|| anyhow::format_err!("expected value to be a sequence"))?
                            .iter()
                            .map(|item| {
                                item.as_str()
                                    .ok_or_else(|| {
                                        anyhow::format_err!("expected sequence item to be a string")
                                    })?
                                    .lines()
                                    .map(|line| line.parse())
                                    .collect::<anyhow::Result<_>>()
                            })
                            .collect::<anyhow::Result<_>>()?,
                    ))
                })
                .collect::<anyhow::Result<_>>()?,
        })
    }
}
