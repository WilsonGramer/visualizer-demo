use clap::Parser;
use std::{fs, io, path::PathBuf};
use wipple::span::ParsedSpan;
use wipple_db::Filter;

#[derive(Parser)]
struct Args {
    path: PathBuf,

    #[clap(short = 'l', long = "line")]
    filter_lines: Vec<u32>,

    #[clap(long)]
    query: Option<String>,

    #[clap(long, requires = "query")]
    query_span: Option<ParsedSpan>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let source = fs::read_to_string(&args.path)?;

    let filter = (!args.filter_lines.is_empty()).then_some(Filter::Lines(&args.filter_lines));

    let options = wipple::Options {
        path: &args.path.display().to_string(),
        source: &source,
        filter,
        queries: Vec::from_iter(args.query.zip(args.query_span)),
        ..Default::default()
    };

    wipple::run(options, io::stdout(), None::<fn(_)>)
}
