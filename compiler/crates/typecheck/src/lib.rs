pub mod constraints;
pub mod context;
mod debug;
pub mod nodes;
pub mod session;

use crate::{constraints::ToConstraintsContext, context::Context, session::Session};
use wipple_compiler_trace::NodeId;

impl Context<'_> {
    pub fn session<'a>(&'a self, filter: impl Fn(NodeId) -> bool + 'a) -> Session<'a> {
        let mut ctx = ToConstraintsContext::new(self);
        ctx.register_all();

        let nodes = self
            .nodes
            .keys()
            .copied()
            .inspect(|&node| {
                ctx.visit(node);
            })
            .collect::<Vec<_>>();

        Session::from_constraints(nodes, ctx.into_constraints(), filter)
    }
}
