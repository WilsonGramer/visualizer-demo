use crate::{Context, selectors::Select};
use wipple_compiler_trace::{NodeId, Rule};

#[derive(Clone)]
pub struct Node<R: Rule>(pub NodeId, pub R);

impl<R: Rule> Select for Node<R> {
    fn select<'a>(ctx: &'a Context, node: NodeId, f: impl Fn(&'a Context, Self)) {
        let Some(rule) = ctx.nodes.get(&node) else {
            ctx.no_results();
            return;
        };

        if !rule.is::<R>() {
            ctx.no_results();
            return;
        }

        f(ctx, Node(node, R::init()))
    }
}
