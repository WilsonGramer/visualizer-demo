use crate::{Context, selectors::Select};

mod prelude {
    pub use crate::Context;
    pub use crate::feedback::*;
    pub use crate::selectors::select::*;
    pub use wipple_compiler_lower::rule as lower;
    pub use wipple_compiler_typecheck::nodes::rule as typecheck;
}

register_queries! {
    mod add_unit;
    mod attribute_errors;
    mod missing_number_type;
    mod missing_text_type;
    mod multiple_types;
    mod placeholder;
    mod unresolved_name;
}

macro_rules! register_queries {
    ($(mod $mod:ident;)*) => {
        $(
            mod $mod;
        )*

        pub fn run(ctx: &Context<'_>) {
            $(
                $mod::run(ctx);
            )*
        }
    };
}

use register_queries;

pub trait Query<'a, S> {
    fn run(&self, ctx: &'a Context<'_>);
}

// Tuple implementations

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, F: Fn(&'a Context<'_>, S1)> Query<'a, (S1,)> for F {
    fn run(&self, ctx: &'a Context<'_>) {
        ctx.select_all::<S1>(|ctx, _, s1| {
            self(ctx, s1);
        });
    }
}

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, S2: Select, F: Fn(&'a Context<'_>, S1, S2)> Query<'a, (S1, S2)> for F {
    fn run(&self, ctx: &'a Context<'_>) {
        ctx.select_all::<S1>(|ctx, _, s1| {
            ctx.select_all::<S2>(|ctx, _, s2| {
                self(ctx, s1.clone(), s2);
            });
        })
    }
}

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, S2: Select, S3: Select, F: Fn(&'a Context<'_>, S1, S2, S3)>
    Query<'a, (S1, S2, S3)> for F
{
    fn run(&self, ctx: &'a Context<'_>) {
        ctx.select_all::<S1>(|ctx, _, s1| {
            ctx.select_all::<S2>(|ctx, _, s2| {
                ctx.select_all::<S3>(|ctx, _, s3| {
                    self(ctx, s1.clone(), s2.clone(), s3);
                });
            });
        });
    }
}
