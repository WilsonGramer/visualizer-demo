mod and;
mod any_rule;
mod child;
mod influence;
mod node;
mod rule;
mod tys;

pub mod select {
    pub use super::and::*;
    pub use super::any_rule::*;
    pub use super::child::*;
    pub use super::influence::*;
    pub use super::node::*;
    pub use super::rule::*;
    pub use super::tys::*;
}

use crate::Context;
use wipple_compiler_trace::NodeId;

pub trait Select: Clone {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self));
}
