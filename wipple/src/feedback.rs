use colored::Colorize;
use std::io::{self, Write};
use visualizer::Ty;
use wipple_db::{Db, FactValue, MarkdownQueryExt, Query};
use wipple_syntax::Parse;

#[derive(rust_embed::RustEmbed)]
#[folder = "feedback"]
struct Feedback;

pub fn write_feedback(db: &Db, mut output: impl Write) -> io::Result<()> {
    let queries = Feedback::iter()
        .filter(|path| path.ends_with(".md"))
        .map(|path| {
            let markdown = String::from_utf8(Feedback::get(&path).unwrap().data.to_vec()).unwrap();

            Query::markdown(&markdown, matcher)
                .unwrap_or_else(|| panic!("invalid feedback file: {path}"))
        });

    for (span, message) in queries.flat_map(|query| query.run(db)) {
        let message = textwrap::wrap(
            &message,
            textwrap::Options::new(80)
                .initial_indent("    ")
                .subsequent_indent("    "),
        )
        .join("\n");

        writeln!(
            output,
            "{}\n\n{}\n",
            format!("{span:?}").bold().underline(),
            message
        )?;
    }

    Ok(())
}

fn matcher(db: &Db, value: &dyn FactValue, s: &str) -> bool {
    if let Some(other) = value.downcast_ref::<String>() {
        return s == other;
    }

    if let Some(ty) = value
        .downcast_ref::<Ty<Db>>()
        .and_then(|ty| TyMatcher::from_ty(ty, db))
        && let Some(other) = wipple_syntax::Type::parse(s)
            .ok()
            .and_then(TyMatcher::from_syntax)
    {
        return ty.unifies_with(&other);
    }

    false
}

enum TyMatcher {
    Placeholder,
    Unit,
    Named(String, Vec<TyMatcher>),
    Block(Box<TyMatcher>),
    Function(Vec<TyMatcher>, Box<TyMatcher>),
    Parameter(String),
    Tuple(Vec<TyMatcher>),
}

impl TyMatcher {
    fn from_ty(ty: &Ty<Db>, db: &Db) -> Option<Self> {
        Some(match ty {
            Ty::Unknown | Ty::Of(_) => TyMatcher::Placeholder,
            Ty::Parameter(parameter) => {
                TyMatcher::Parameter(db.get::<String>(*parameter, "source")?.clone())
            }
            Ty::Named { name, parameters } => TyMatcher::Named(
                db.get::<String>(*name, "source")?.clone(),
                parameters
                    .values()
                    .filter_map(|ty| TyMatcher::from_ty(ty, db))
                    .collect(),
            ),
            Ty::Function { inputs, output } => TyMatcher::Function(
                inputs
                    .iter()
                    .filter_map(|ty| TyMatcher::from_ty(ty, db))
                    .collect(),
                Box::new(TyMatcher::from_ty(output, db)?),
            ),
            Ty::Tuple { elements } => TyMatcher::Tuple(
                elements
                    .iter()
                    .filter_map(|ty| TyMatcher::from_ty(ty, db))
                    .collect(),
            ),
        })
    }

    fn from_syntax(ty: wipple_syntax::Type) -> Option<Self> {
        Some(match ty {
            wipple_syntax::Type::Placeholder(_) => TyMatcher::Placeholder,
            wipple_syntax::Type::Unit(_) => TyMatcher::Unit,
            wipple_syntax::Type::Named(ty) => TyMatcher::Named(ty.name.value.clone(), Vec::new()),
            wipple_syntax::Type::Parameterized(ty) => TyMatcher::Named(
                ty.name.value.clone(),
                ty.parameters
                    .into_iter()
                    .filter_map(|ty| TyMatcher::from_syntax(ty.0))
                    .collect(),
            ),
            wipple_syntax::Type::Block(ty) => {
                TyMatcher::Block(Box::new(TyMatcher::from_syntax(*ty.output)?))
            }
            wipple_syntax::Type::Function(ty) => TyMatcher::Function(
                ty.inputs
                    .0
                    .into_iter()
                    .filter_map(TyMatcher::from_syntax)
                    .collect(),
                Box::new(TyMatcher::from_syntax(*ty.output)?),
            ),
            wipple_syntax::Type::Parameter(ty) => TyMatcher::Parameter(ty.name.value.clone()),
            wipple_syntax::Type::Tuple(ty) => TyMatcher::Tuple(
                ty.elements
                    .into_iter()
                    .filter_map(TyMatcher::from_syntax)
                    .collect(),
            ),
        })
    }

    fn unifies_with(&self, other: &Self) -> bool {
        match (self, other) {
            (TyMatcher::Placeholder, _) | (_, TyMatcher::Placeholder) => true,
            (TyMatcher::Unit, TyMatcher::Unit) => true,
            (
                TyMatcher::Named(left_name, left_params),
                TyMatcher::Named(right_name, right_params),
            ) => {
                left_name == right_name
                    && left_params.len() == right_params.len()
                    && left_params
                        .iter()
                        .zip(right_params.iter())
                        .all(|(left, right)| left.unifies_with(right))
            }
            (TyMatcher::Block(left_output), TyMatcher::Block(right_output)) => {
                left_output.unifies_with(right_output)
            }
            (
                TyMatcher::Function(left_inputs, left_output),
                TyMatcher::Function(right_inputs, right_output),
            ) => {
                left_inputs.len() == right_inputs.len()
                    && left_inputs
                        .iter()
                        .zip(right_inputs.iter())
                        .all(|(left, right)| left.unifies_with(right))
                    && left_output.unifies_with(right_output)
            }
            (TyMatcher::Parameter(left_name), TyMatcher::Parameter(right_name)) => {
                left_name == right_name
            }
            (TyMatcher::Tuple(left_elements), TyMatcher::Tuple(right_elements)) => {
                left_elements.len() == right_elements.len()
                    && left_elements
                        .iter()
                        .zip(right_elements.iter())
                        .all(|(left, right)| left.unifies_with(right))
            }
            _ => false,
        }
    }
}
