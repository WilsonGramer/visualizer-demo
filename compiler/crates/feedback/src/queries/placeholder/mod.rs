use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>,
         And(
            And(And(Node(placeholder), Rule(lower::placeholder)), Ty(ty::Any(ty))),
            And(Influence(influence), AnyRule(rule)),
        )| {
            ctx.feedback(
                TEMPLATE
                    .clone()
                    .node("placeholder", placeholder)
                    .ty("ty", ty)
                    .node("influence", influence)
                    .rule("rule", rule),
            );
        },
    );
}
