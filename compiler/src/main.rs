use colored::Colorize;
use std::{
    env, fs,
    io::{self, Write},
    sync::Arc,
};
use wipple_compiler_trace::Span;
use wipple_compiler_typecheck::context::DebugOptions;

fn main() {
    match env::args().nth(1) {
        Some(path) => {
            let source = fs::read_to_string(&path).unwrap();
            run(&path, &source);
        }
        None => loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut source = String::new();
            loop {
                if io::stdin().read_line(&mut source).unwrap() <= 1 {
                    break;
                }
            }

            run("stdin", &source);
        },
    };
}

fn run(path: &str, source: &str) {
    let display_syntax_error = |error| eprintln!("syntax error: {error:?}");

    let display_graph = |graph: String| {
        let mut process = std::process::Command::new("sh")
            .arg("-c")
            .arg("dot -Tpng -Gdpi=200 | imgcat")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        process
            .stdin
            .as_ref()
            .unwrap()
            .write_all(graph.as_bytes())
            .unwrap();

        process.wait().unwrap();
    };

    let display_tys = |tys| println!("{tys}");

    wipple_compiler::compile(
        path,
        source,
        display_syntax_error,
        display_graph,
        display_tys,
    );
}
