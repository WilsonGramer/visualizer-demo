mod and;
mod node;
mod tys;

pub mod select {
    pub use super::and::*;
    pub use super::node::*;
    pub use super::tys::*;
}

use crate::Context;
use wipple_compiler_trace::NodeId;

pub trait Select: Clone {
    fn select<'a>(ctx: &'a Context, node: NodeId, f: impl Fn(&'a Context, Self));
}
