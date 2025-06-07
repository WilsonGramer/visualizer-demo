use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>, And(Node(literal), Rule(lower::missing_text_type))| {
            ctx.feedback(TEMPLATE.clone().node("literal", literal));
        },
    );
}
