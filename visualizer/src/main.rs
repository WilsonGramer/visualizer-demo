use clap::Parser;
use std::{fs, io, path::PathBuf};

#[derive(Parser)]
struct Args {
    path: PathBuf,

    #[clap(short, long)]
    graph: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let source = fs::read_to_string(&args.path)?;

    clearscreen::clear().unwrap();

    let mut mermaid_process = args
        .graph
        .map(|graph| {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("mmdc -i - -o {} --scale 3", graph.display()))
                .stdin(std::process::Stdio::piped())
                .spawn()
        })
        .transpose()?;

    let result = wipple_visualizer::compile(
        &args.path.display().to_string(),
        &source,
        io::stdout(),
        mermaid_process.as_mut().map(|p| p.stdin.as_mut().unwrap()),
    );

    if let Some(mut process) = mermaid_process {
        process.wait()?;
    }

    result
}
