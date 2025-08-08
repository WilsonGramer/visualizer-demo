use crate::matcher::matcher;
use std::{collections::HashMap, rc::Rc, sync::LazyLock};
use wipple_db::{Db, Query, QueryValues, Span, YamlQueryExt};

static QUERIES: LazyLock<HashMap<String, Query<'static, Vec<QueryValues>>>> = LazyLock::new(|| {
    Query::yaml(include_str!("../queries.yml"), &matcher).expect("failed to parse actions")
});

pub fn run_query(name: &str, db: &Db, input: Span) -> anyhow::Result<Vec<Span>> {
    let query = QUERIES
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("no such query '{name}'"))?;

    let values = query.run(
        db,
        HashMap::from([(String::from("input"), Rc::new(input) as _)]),
    );

    Ok(values
        .into_iter()
        .filter_map(|mut values| {
            values
                .remove("output")
                .and_then(|value| value.downcast_ref::<Span>().cloned())
        })
        .collect())
}
