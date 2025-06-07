use super::prelude::*;

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>, And(Node(attribute), Rule(lower::unknown_attribute))| {
            todo!();
        },
    );

    ctx.run(
        |ctx: &Context<'_>, And(Node(attribute), Rule(lower::duplicate_attribute))| {
            todo!();
        },
    );

    ctx.run(
        |ctx: &Context<'_>, And(Node(attribute), Rule(lower::missing_attribute_value))| {
            todo!();
        },
    );

    ctx.run(
        |ctx: &Context<'_>, And(Node(attribute), Rule(lower::extra_attribute_value))| {
            todo!();
        },
    );
}
