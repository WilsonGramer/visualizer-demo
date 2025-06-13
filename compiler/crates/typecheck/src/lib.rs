pub mod constraints;
pub mod context;
pub mod nodes;
pub mod session;

use crate::{constraints::ToConstraintsContext, context::Context, session::Session};

impl Context<'_> {
    pub fn session(&self) -> Session {
        let mut ctx = ToConstraintsContext::new(self);
        ctx.register_all();

        for node in self.nodes() {
            ctx.visit(node);
        }

        Session::from_constraints(self.nodes(), ctx.into_constraints())
    }
}
