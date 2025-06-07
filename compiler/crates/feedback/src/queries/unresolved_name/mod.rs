use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>, And(Node(name), Rule(lower::unresolved_name))| {
            ctx.feedback(TEMPLATE.clone().node("name", name));
        },
    );
}
