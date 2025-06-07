mod and;
mod child;
mod node;
mod rule;
mod tys;

pub mod select {
    pub use super::and::*;
    pub use super::child::*;
    pub use super::node::*;
    pub use super::rule::*;
    pub use super::tys::*;
}

use crate::Context;
use wipple_compiler_trace::NodeId;

pub trait Select: Clone {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self));
}
