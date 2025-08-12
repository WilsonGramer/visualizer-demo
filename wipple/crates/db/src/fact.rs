use crate::Db;
use dyn_eq::DynEq;
use std::{any::Any, fmt::Debug, rc::Rc};
use visualizer::{Substitutions, Ty};

#[derive(Debug, Clone)]
pub struct Fact {
    pub(crate) name: Rc<str>,
    pub(crate) value: Rc<dyn FactValue>,
}

pub trait FactValue: Any + Debug + DynEq + Send + Sync {
    fn display(&self, db: &Db) -> Option<String>;

    fn is_code(&self) -> bool {
        false
    }
}

dyn_eq::eq_trait_object!(FactValue);

impl dyn FactValue {
    pub fn is<T: 'static>(&self) -> bool {
        (self as &dyn Any).is::<T>()
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

impl Fact {
    pub fn new(name: impl AsRef<str>, value: impl FactValue) -> Self {
        Fact {
            name: Rc::from(name.as_ref()),
            value: Rc::new(value),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &dyn FactValue {
        self.value.as_ref()
    }

    pub fn clone_value(&self) -> Rc<dyn FactValue> {
        self.value.clone()
    }
}

impl FactValue for () {
    fn display(&self, _db: &Db) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Source(pub String);

impl FactValue for Source {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(self.0.clone())
    }
}

impl FactValue for Ty<Db> {
    fn display(&self, db: &Db) -> Option<String> {
        Some(display_ty(self, db, true))
    }

    fn is_code(&self) -> bool {
        true
    }
}

fn display_ty(ty: &Ty<Db>, db: &Db, root: bool) -> String {
    match ty {
        Ty::Unknown(_) | Ty::Of(_) => String::from("_"),
        Ty::Parameter(node) => db.get::<Source>(*node, "source").unwrap().clone().0,
        Ty::Named { name, parameters } => {
            let ty = format!(
                "{}{}",
                db.get::<Source>(*name, "source").unwrap().0,
                parameters
                    .values()
                    .map(|parameter| format!(" {}", display_ty(parameter, db, false)))
                    .collect::<String>()
            );

            if root || parameters.is_empty() {
                ty
            } else {
                format!("({})", ty)
            }
        }
        Ty::Function { inputs, output } => {
            let ty = format!(
                "{}-> {}",
                inputs
                    .iter()
                    .map(|input| format!("{} ", display_ty(input, db, false)))
                    .collect::<String>(),
                display_ty(output, db, true)
            );

            if root { ty } else { format!("({})", ty) }
        }
        Ty::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(|element| display_ty(element, db, false))
                .collect::<Vec<_>>()
                .join(" ; ")
        ),
    }
}

impl FactValue for Substitutions<Db> {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(String::from("Substitutions(..)"))
    }
}

impl Fact {
    pub fn hidden() -> Self {
        Fact::new("hidden", ())
    }

    pub fn is_hidden(&self) -> bool {
        self.name() == "hidden"
    }
}
