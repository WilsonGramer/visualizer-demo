use crate::{Context, selectors::Select};
use wipple_compiler_trace::NodeId;

#[derive(Clone)]
pub struct And<S1: Select, S2: Select>(pub S1, pub S2);

impl<S1: Select, S2: Select> Select for And<S1, S2> {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        S1::select(ctx, node, |ctx, node, s1| {
            S2::select(ctx, node, |ctx, node, s2| f(ctx, node, And(s1.clone(), s2)));
        });
    }
}
