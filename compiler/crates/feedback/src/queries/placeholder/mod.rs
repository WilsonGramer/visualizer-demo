use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>,
         And(And(Node(placeholder), Rule(lower::placeholder)), Ty(ty::Any(ty)))| {
            ctx.feedback(
                TEMPLATE
                    .clone()
                    .node("placeholder", placeholder)
                    .ty("ty", ty),
            );
        },
    );
}
