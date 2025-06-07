use super::prelude::*;

include_template!("template.yml");

pub fn run(ctx: &Context<'_>) {
    ctx.run(
        |ctx: &Context<'_>,
         And(
            Child(call_for_number, number, lower::function_in_function_call),
            Rule(lower::number),
        ),
         Child(call_for_function, function, lower::input_in_function_call)| {
            if call_for_number == call_for_function {
                ctx.feedback(
                    TEMPLATE
                        .clone()
                        .node("number", number)
                        .node("function", function),
                );
            }
        },
    );
}
