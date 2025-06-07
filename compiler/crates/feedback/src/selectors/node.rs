use crate::{Context, selectors::Select};
use wipple_compiler_trace::NodeId;

#[derive(Clone)]
pub struct Node(pub NodeId);

impl Select for Node {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        f(ctx, node, Node(node));
    }
}
