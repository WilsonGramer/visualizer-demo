use crate::{Context, selectors::Select};
use wipple_compiler_trace::{NodeId, Rule as RuleType};

#[derive(Clone)]
pub struct Child<R: RuleType>(pub NodeId, pub NodeId, pub R);

impl<R: RuleType> Select for Child<R> {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        if let Some(relations) = ctx.relations.get(&node) {
            for &(parent, rule) in relations {
                if rule.is::<R>() {
                    f(ctx, node, Child(parent, node, R::init()));
                }
            }
        }
    }
}
