use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(source: String, filter: Option<Vec<u32>>) -> Vec<String> {
    console_error_panic_hook::set_once();
    colored::control::set_override(true);

    let filter = filter
        .and_then(|filter| filter.into_iter().collect_tuple())
        .map(|(start, end)| wipple::db::Filter::Range(start, end));

    let mut output = Vec::new();
    let mut graph = Vec::new();
    wipple::run("input", &source, filter, &mut output, Some(&mut graph)).unwrap();

    vec![
        String::from_utf8(output).unwrap(),
        String::from_utf8(graph).unwrap(),
    ]
}
