use crate::{Context, selectors::select::*};
use wipple_compiler_lower::rule as lower;

pub fn run(ctx: &mut Context) {
    ctx.run(|ctx, Rule(Node(literal), lower::missing_number_type)| {
        todo!();
    });

    ctx.run(|ctx, Rule(Node(literal), lower::missing_text_type)| {
        todo!();
    });
}
