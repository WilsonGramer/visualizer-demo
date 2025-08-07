use crate::{
    File,
    parser::{Arg, Term},
};
use std::{borrow::Cow, collections::HashMap};
use wipple_db::{Db, FactValue, NodeId};

impl File {
    pub fn query<'a>(
        &self,
        db: &'a Db,
        matches: impl Fn(&dyn FactValue, &str) -> bool,
    ) -> impl Iterator<Item = HashMap<String, &'a dyn FactValue>> + 'a {
        let mut result = Vec::new();
        query_inner(
            db,
            &matches,
            &self.terms,
            &HashMap::new(),
            &HashMap::new(),
            &mut result,
        );

        result.into_iter()
    }
}

fn query_inner<'a>(
    db: &'a Db,
    matches: &dyn Fn(&dyn FactValue, &str) -> bool,
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
                            if values.get(variable).is_some_and(|other| !other.eq(value)) {
                                continue;
                            }

                            let mut values = values.clone();
                            values.insert(variable.clone(), value);
                            Cow::Owned(values)
                        }
                        Arg::Value(pattern) => {
                            if !matches(value, pattern) {
                                continue;
                            }

                            Cow::Borrowed(values)
                        }
                    }
                } else {
                    Cow::Borrowed(values)
                };

                query_inner(db, matches, terms, &nodes, &values, result);
            }
        }
        None => result.push(values.clone()),
    }
}
