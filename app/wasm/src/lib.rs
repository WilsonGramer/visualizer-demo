use itertools::Itertools;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(source: String, filter: Option<Vec<u32>>) -> Vec<JsValue> {
    console_error_panic_hook::set_once();
    colored::control::set_override(true);

    let filter = filter
        .and_then(|filter| filter.into_iter().collect_tuple())
        .map(|(start, end)| wipple::db::Filter::Range(start, end));

    let options = wipple::Options {
        path: "input",
        source: &source,
        filter,
        ..Default::default()
    };

    let mut output = Vec::new();
    let mut graph = None;
    wipple::run(options, &mut output, Some(|g| graph = Some(g))).unwrap();

    vec![
        String::from_utf8(output).unwrap().into(),
        graph
            .map(|graph| {
                graph
                    .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                    .unwrap()
            })
            .unwrap_or(JsValue::UNDEFINED),
    ]
}
