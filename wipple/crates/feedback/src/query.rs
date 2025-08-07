use crate::{File, parser::Term};
use std::{borrow::Cow, collections::HashMap};
use wipple_db::{Db, FactValue, NodeId};

impl File {
    pub fn query<'a>(
        &self,
        db: &'a Db,
    ) -> impl Iterator<Item = HashMap<String, &'a dyn FactValue>> + 'a {
        let mut result = Vec::new();
        query_inner(
            db,
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
                let values = if let Some(key) = &next.key {
                    let value = fact.value();

                    if values.get(key).is_some_and(|other| !other.eq(value)) {
                        continue;
                    }

                    let mut values = values.clone();
                    values.insert(key.clone(), value);
                    Cow::Owned(values)
                } else {
                    Cow::Borrowed(values)
                };

                query_inner(db, terms, &nodes, &values, result);
            }
        }
        None => result.push(values.clone()),
    }
}
