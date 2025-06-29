pub mod constraints;
pub mod context;
pub mod debug;
pub mod nodes;
pub mod typechecker;

use crate::{
    constraints::{Constraints, ToConstraintsContext},
    context::Context,
};
use std::collections::BTreeSet;
use wipple_compiler_trace::NodeId;

impl Context<'_> {
    pub fn as_constraints(&self, filter: impl Fn(NodeId) -> bool) -> Constraints {
        let mut ctx = ToConstraintsContext::new(self);
        ctx.register_all();

        let nodes = self
            .nodes
            .keys()
            .copied()
            .filter(|&node| filter(node))
            .inspect(|&node| {
                ctx.visit(node);
            })
            .collect::<BTreeSet<_>>();

        let mut constraints = ctx.into_constraints();
        constraints.nodes.extend(nodes);
        constraints
    }
}
