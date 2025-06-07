use crate::{Context, selectors::select::*};
use wipple_compiler_lower::rule as lower;

pub fn run(ctx: &mut Context) {
    ctx.run(|ctx, Rule(Node(name), lower::unresolved_name)| {
        todo!();
    });
}
