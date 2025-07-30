pub mod constraints;
pub mod debug;
pub mod feedback;
pub mod nodes;
pub mod typechecker;

use crate::{
    constraints::{Constraint, ToConstraintsContext},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

pub fn collect_constraints<'a>(
    nodes: impl IntoIterator<Item = (NodeId, &'a dyn Node)>,
) -> Vec<Constraint> {
    let mut ctx = ToConstraintsContext::default();
    ctx.register_all();

    for (id, node) in nodes {
        ctx.visit(id, node);
    }

    ctx.into_constraints()
}
