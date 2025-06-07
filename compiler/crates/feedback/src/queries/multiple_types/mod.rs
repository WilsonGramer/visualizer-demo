use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>, And(And(Node(node), Ty(ty::Any(left))), Ty(ty::Any(right)))| {
            if left == right {
                return;
            }

            ctx.feedback(
                TEMPLATE
                    .clone()
                    .node("node", node)
                    .ty("left", left)
                    .ty("right", right),
            );
        },
    );
}
