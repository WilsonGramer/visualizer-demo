use crate::{Context, selectors::Select};
use wipple_compiler_trace::NodeId;

#[derive(Clone)]
pub struct Influence(pub NodeId);

impl Select for Influence {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        let Some((_, influences)) = ctx.tys.get(&node) else {
            return;
        };

        let influences = influences.iter().map(|(node, _)| *node).collect::<Vec<_>>();

        for node in influences {
            f(ctx, node, Influence(node));
        }
    }
}
