pub mod constraints;
pub mod context;
mod debug;
pub mod nodes;
pub mod typechecker;

use crate::{constraints::ToConstraintsContext, context::Context, typechecker::Typechecker};
use wipple_compiler_trace::NodeId;

impl Context<'_> {
    pub fn typechecker_from_constraints_where<'a>(
        &'a self,
        filter: impl Fn(NodeId) -> bool + 'a,
    ) -> Typechecker<'a> {
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

        Typechecker::from_constraints(nodes, ctx.into_constraints(), filter)
    }
}
