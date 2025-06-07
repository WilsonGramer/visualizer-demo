use crate::{Context, selectors::select::*};
use wipple_compiler_lower::rule as lower;
use wipple_compiler_typecheck::nodes::rule as typecheck;

pub fn run(ctx: &mut Context) {
    ctx.run(
        |ctx,
         Rule(
            Rule(
                Rule(Ty(Node(literal), ty::Named(literal_ty_name, _)), typecheck::annotated),
                typecheck::function,
            ),
            lower::function_in_function_call,
        ),
         Rule(
            Rule(Ty(Node(function), ty::Function(inputs, output)), typecheck::call),
            lower::input_in_function_call,
        )| {
            // if literal_ty_name is "number"
            // add_feedback("add `[unit]` to this function")

            todo!();
        },
    );
}
