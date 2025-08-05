use crate::{DisplayProvider, NodeId, Span, Ty};
use std::{any::Any, fmt::Debug, rc::Rc};

#[derive(Debug, Clone)]
pub struct Fact {
    namespace: Rc<str>,
    name: Rc<str>,
    value: Rc<dyn FactValue>,
}

pub trait FactValue: Any + Debug {
    fn display(&self, provider: &dyn DisplayProvider) -> Option<String>;

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
    pub fn new(namespace: impl AsRef<str>, name: impl AsRef<str>, value: impl FactValue) -> Self {
        Fact {
            namespace: Rc::from(namespace.as_ref()),
            name: Rc::from(name.as_ref()),
            value: Rc::new(value),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &dyn FactValue {
        self.value.as_ref()
    }
}

impl FactValue for () {
    fn display(&self, _provider: &dyn DisplayProvider) -> Option<String> {
        None
    }
}

impl FactValue for String {
    fn display(&self, _provider: &dyn DisplayProvider) -> Option<String> {
        Some(self.clone())
    }
}

impl FactValue for NodeId {
    fn display(&self, _provider: &dyn DisplayProvider) -> Option<String> {
        Some(format!("{self:?}"))
    }
}

impl FactValue for Span {
    fn display(&self, _provider: &dyn DisplayProvider) -> Option<String> {
        Some(format!("{self:?}"))
    }
}

impl FactValue for Ty {
    fn display(&self, provider: &dyn DisplayProvider) -> Option<String> {
        Some(self.display(provider))
    }

    fn is_code(&self) -> bool {
        true
    }
}
