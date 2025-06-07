use crate::{Context, selectors::Select};
use wipple_compiler_trace::{NodeId, Rule as RuleType};

#[derive(Clone)]
pub struct Rule<R: RuleType>(pub R);

impl<R: RuleType> Select for Rule<R> {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        let Some(rule) = ctx.nodes.get(&node) else {
            return;
        };

        if rule.is::<R>() {
            f(ctx, node, Rule(R::init()));
        }
    }
}
