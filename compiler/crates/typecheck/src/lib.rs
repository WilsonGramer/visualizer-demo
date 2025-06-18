pub mod constraints;
pub mod context;
mod debug;
pub mod nodes;
pub mod session;

use crate::{constraints::ToConstraintsContext, context::Context, session::Session};

impl Context<'_> {
    pub fn session(&self) -> Session {
        let mut ctx = ToConstraintsContext::new(self);
        ctx.register_all();

        let nodes = self
            .nodes()
            .map(|(node, _)| {
                ctx.visit(node);
                node
            })
            .collect::<Vec<_>>();

        Session::from_constraints(nodes, ctx.into_constraints())
    }
}
