use crate::{Context, selectors::Select};
use wipple_compiler_trace::{AnyRule as AnyRuleType, NodeId};

#[derive(Clone)]
pub struct AnyRule(pub AnyRuleType);

impl Select for AnyRule {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        let Some(rule) = ctx.nodes.get(&node) else {
            return;
        };

        f(ctx, node, AnyRule(*rule));
    }
}
