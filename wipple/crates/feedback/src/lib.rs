mod parser;
mod query;

pub use parser::File;

use itertools::Itertools;
use visualizer::Ty;
use wipple_db::{Db, FactValue, Span};
use wipple_syntax::{Parse, Type};

pub fn iter_feedback<'a>(
    db: &Db,
    files: impl IntoIterator<Item = &'a File>,
) -> impl Iterator<Item = (Span, String)> {
    let mut feedback = Vec::new();
    for file in files {
        for values in file.query(db, |value, s| matches(value, s, db)) {
            let mut body = file.body.clone();
            for link in file.links.iter().rev() {
                if let Some(value) = values.get(&link.name).and_then(|value| value.display(db)) {
                    body.replace_range(
                        link.range.clone(),
                        &if link.code {
                            format!("`{value}`")
                        } else {
                            value
                        },
                    );
                }
            }

            if let Some(span) = values
                .get("span")
                .and_then(|value| value.downcast_ref::<Span>())
            {
                feedback.push((span.clone(), body));
            }
        }
    }

    feedback.into_iter().unique()
}

fn matches(value: &dyn FactValue, s: &str, db: &Db) -> bool {
    if let Some(other) = value.downcast_ref::<String>() {
        return s == other;
    }

    if let Some(ty) = value
        .downcast_ref::<Ty<Db>>()
        .and_then(|ty| TyMatcher::from_ty(ty, db))
        && let Some(other) = Type::parse(s).ok().and_then(TyMatcher::from_syntax)
    {
        return ty.unify(&other);
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

    fn from_syntax(ty: Type) -> Option<Self> {
        Some(match ty {
            Type::Placeholder(_) => TyMatcher::Placeholder,
            Type::Unit(_) => TyMatcher::Unit,
            Type::Named(ty) => TyMatcher::Named(ty.name.value.clone(), Vec::new()),
            Type::Parameterized(ty) => TyMatcher::Named(
                ty.name.value.clone(),
                ty.parameters
                    .into_iter()
                    .filter_map(|ty| TyMatcher::from_syntax(ty.0))
                    .collect(),
            ),
            Type::Block(ty) => TyMatcher::Block(Box::new(TyMatcher::from_syntax(*ty.output)?)),
            Type::Function(ty) => TyMatcher::Function(
                ty.inputs
                    .0
                    .into_iter()
                    .filter_map(TyMatcher::from_syntax)
                    .collect(),
                Box::new(TyMatcher::from_syntax(*ty.output)?),
            ),
            Type::Parameter(ty) => TyMatcher::Parameter(ty.name.value.clone()),
            Type::Tuple(ty) => TyMatcher::Tuple(
                ty.elements
                    .into_iter()
                    .filter_map(TyMatcher::from_syntax)
                    .collect(),
            ),
        })
    }

    fn unify(&self, other: &Self) -> bool {
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
                        .all(|(left, right)| left.unify(right))
            }
            (TyMatcher::Block(left_output), TyMatcher::Block(right_output)) => {
                left_output.unify(right_output)
            }
            (
                TyMatcher::Function(left_inputs, left_output),
                TyMatcher::Function(right_inputs, right_output),
            ) => {
                left_inputs.len() == right_inputs.len()
                    && left_inputs
                        .iter()
                        .zip(right_inputs.iter())
                        .all(|(left, right)| left.unify(right))
                    && left_output.unify(right_output)
            }
            (TyMatcher::Parameter(left_name), TyMatcher::Parameter(right_name)) => {
                left_name == right_name
            }
            (TyMatcher::Tuple(left_elements), TyMatcher::Tuple(right_elements)) => {
                left_elements.len() == right_elements.len()
                    && left_elements
                        .iter()
                        .zip(right_elements.iter())
                        .all(|(left, right)| left.unify(right))
            }
            _ => false,
        }
    }
}
