use crate::{feedback::Feedback, selectors::Selector};
use include_dir::include_dir;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub r#as: Option<String>,

    pub rule: Option<String>,

    #[serde(rename = "where", default)]
    pub selectors: Vec<Selector>,

    #[serde(flatten)]
    pub item: Feedback,
}

pub static QUERIES: LazyLock<Vec<(String, Query)>> = LazyLock::new(|| {
    include_dir!("$CARGO_MANIFEST_DIR/queries/json")
        .find("**/*.json")
        .unwrap()
        .map(|entry| {
            let name = entry.path().with_extension("").display().to_string();

            let contents = entry.as_file().unwrap().contents_utf8().unwrap();

            let query = serde_json::from_str::<Query>(contents)
                .unwrap_or_else(|err| panic!("invalid template content in {name}: {err}"));

            (name, query)
        })
        .collect()
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn generate_json_schemas() {
        let target = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let json = serde_json::to_string_pretty(&schemars::schema_for!(Query)).unwrap();
        std::fs::write(target.join("queries/_schema.json"), json).unwrap();
    }
}
