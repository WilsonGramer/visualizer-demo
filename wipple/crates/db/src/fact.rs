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

impl FactValue for String {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(self.clone())
    }
}

impl FactValue for u32 {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(self.to_string())
    }
}

impl FactValue for Ty<Db> {
    fn display(&self, db: &Db) -> Option<String> {
        Some(match self {
            Ty::Unknown | Ty::Of(_) => String::from("_"),
            Ty::Parameter(node) => db.get::<String>(*node, "source").unwrap().clone(),
            Ty::Named { name, parameters } => format!(
                "{}{}",
                db.get::<String>(*name, "source").unwrap(),
                parameters
                    .values()
                    .map(|parameter| format!(" {}", parameter.display(db).unwrap()))
                    .collect::<String>()
            ),
            Ty::Function { inputs, output } => format!(
                "{}-> {}",
                inputs
                    .iter()
                    .map(|input| format!("{} ", input.display(db).unwrap()))
                    .collect::<String>(),
                output.display(db).unwrap()
            ),
            Ty::Tuple { elements } => format!(
                "({})",
                elements
                    .iter()
                    .map(|element| element.display(db).unwrap())
                    .collect::<Vec<_>>()
                    .join(" ; ")
            ),
        })
    }

    fn is_code(&self) -> bool {
        true
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
