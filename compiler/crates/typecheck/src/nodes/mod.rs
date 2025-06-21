use crate::constraints::ToConstraintsContext;
use std::{any::Any, fmt::Debug};

pub trait Node: Debug + Any {
    fn boxed(self) -> Box<dyn Node>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl Node for Box<dyn Node> {
    fn boxed(self) -> Box<dyn Node>
    where
        Self: Sized,
    {
        // Don't box again, since that changes the `TypeId`
        self
    }
}

impl dyn Node {
    pub fn downcast<T: Node>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
}

register_nodes! {
    mod block => BlockNode;
    mod call => CallNode;
    mod constraint => ConstraintNode;
    mod function => FunctionNode;
    mod placeholder => PlaceholderNode;
    mod tuple_element => TupleElementNode;
}

macro_rules! register_nodes {
    ($(mod $mod:ident => $node:ident;)*) => {
        $(
            mod $mod;
            pub use $mod::$node;
        )*

        impl ToConstraintsContext<'_> {
            pub fn register_all(&mut self) {
                $(
                    self.register::<$node>();
                )*
            }
        }
    };
}

use register_nodes;
