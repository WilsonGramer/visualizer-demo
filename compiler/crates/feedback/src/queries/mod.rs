use crate::{Context, selectors::Select};
use wipple_compiler_trace::NodeId;

register_queries! {
    // mod add_unit;
    // mod attribute_errors;
    // mod missing_literal_type;
    // mod placeholder;
    // mod unresolved_name;
}

macro_rules! register_queries {
    ($(mod $mod:ident;)*) => {
        $(
            mod $mod;
        )*

        pub fn run(ctx: &mut Context) {
            $(
                $mod::run(ctx);
            )*
        }
    };
}

use register_queries;

/// Each selector in `S` will only match nodes related to the ones before it
pub trait Query<'a, S> {
    fn run(&self, ctx: &'a Context, node: NodeId);
}

// Tuple implementations

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, F: Fn(&'a Context, S1)> Query<'a, (S1,)> for F {
    fn run(&self, ctx: &'a Context, node: NodeId) {
        S1::select(ctx, node, |ctx, s| {
            self(ctx, s);
        });
    }
}

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, S2: Select, F: Fn(&'a Context, S1, S2)> Query<'a, (S1, S2)> for F {
    fn run(&self, ctx: &'a Context, node: NodeId) {
        S1::select(ctx, node, |ctx, s1| {
            S2::select(ctx, node, |ctx, s2| {
                self(ctx, s1.clone(), s2);
            });
        })
    }
}

#[diagnostic::do_not_recommend]
impl<'a, S1: Select, S2: Select, S3: Select, F: Fn(&'a Context, S1, S2, S3)> Query<'a, (S1, S2, S3)>
    for F
{
    fn run(&self, ctx: &'a Context, node: NodeId) {
        S1::select(ctx, node, |ctx, s1| {
            S2::select(ctx, node, |ctx, s2| {
                S3::select(ctx, node, |ctx, s3| {
                    self(ctx, s1.clone(), s2.clone(), s3);
                })
            });
        })
    }
}
