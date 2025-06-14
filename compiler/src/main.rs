use std::{
    env, fs,
    io::{self, Write},
    sync::LazyLock,
};

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

static HIGHLIGHT: LazyLock<Box<dyn Fn(&str) + Send + Sync>> = LazyLock::new(|| {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

    let syntax = SyntaxSet::load_defaults_newlines();

    let theme = ThemeSet::load_from_reader(&mut std::io::Cursor::new(
        if terminal_light::luma().is_ok_and(|luma| luma > 0.6) {
            include_str!("themes/GitHub Light.tmTheme")
        } else {
            include_str!("themes/GitHub Dark.tmTheme")
        },
    ))
    .unwrap();

    Box::new(move |s| {
        let mut highlight =
            HighlightLines::new(syntax.find_syntax_by_extension("md").unwrap(), &theme);

        for line in LinesWithEndings::from(s) {
            let ranges = highlight.highlight_line(line, &syntax).unwrap();
            print!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
        }

        println!("\x1b[0m"); // reset color
    })
});

fn print_highlighted(source: impl AsRef<str>) {
    (HIGHLIGHT)(source.as_ref());
    io::stdout().flush().unwrap();
}

fn run(path: &str, source: &str) {
    let display_syntax_error = |error| {
        print_highlighted(format!("syntax error: {error:?}"));
    };

    let display_graph = |graph: String| {
        let mut process = std::process::Command::new("sh")
            .arg("-c")
            .arg("mmdc -i - -o - -e png --scale 3 | imgcat")
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

    let display_feedback = |feedback| {
        println!();
        print_highlighted(feedback);
    };

    wipple_compiler::compile(
        path,
        source,
        display_syntax_error,
        display_graph,
        display_tys,
        display_feedback,
    );
}
