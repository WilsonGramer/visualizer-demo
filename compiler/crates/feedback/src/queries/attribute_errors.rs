use crate::{Context, selectors::select::*};
use wipple_compiler_lower::rule as lower;

pub fn run(ctx: &mut Context) {
    ctx.run(|ctx, Rule(Node(attribute), lower::unknown_attribute)| {
        todo!();
    });

    ctx.run(|ctx, Rule(Node(attribute), lower::duplicate_attribute)| {
        todo!();
    });

    ctx.run(
        |ctx, Rule(Node(attribute), lower::missing_attribute_value)| {
            todo!();
        },
    );

    ctx.run(|ctx, Rule(Node(attribute), lower::extra_attribute_value)| {
        todo!();
    });
}
