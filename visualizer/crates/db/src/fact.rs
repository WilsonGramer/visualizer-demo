use crate::Db;
use std::{any::Any, fmt::Debug, rc::Rc};

#[derive(Debug, Clone)]
pub struct Fact {
    pub(crate) name: Rc<str>,
    pub(crate) value: Rc<dyn FactValue>,
}

pub trait FactValue: Any + Debug {
    fn display(&self, db: &Db) -> Option<String>;

    fn is_code(&self) -> bool {
        false
    }
}

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

impl Fact {
    pub fn hidden() -> Self {
        Fact::new("hidden", ())
    }

    pub fn is_hidden(&self) -> bool {
        self.name() == "hidden"
    }
}
