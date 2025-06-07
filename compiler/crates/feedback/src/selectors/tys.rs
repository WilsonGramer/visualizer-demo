use crate::{Context, selectors::Select};
use wipple_compiler_trace::{AnyRule, NodeId};
use wipple_compiler_typecheck::constraints::Ty as TyType;

#[derive(Clone)]
pub struct Ty<S: Select, T: ty::Select>(pub S, pub T, pub AnyRule);

impl<S: Select, T: ty::Select> Select for Ty<S, T> {
    fn select<'a>(ctx: &'a Context, node: NodeId, f: impl Fn(&'a Context, Self)) {
        let Some(tys) = ctx.tys.get(&node) else {
            ctx.no_results();
            return;
        };

        for ty in tys {}
    }
}

#[allow(clippy::module_inception)]
pub mod ty {
    use super::*;

    pub trait Select: Clone {
        fn select_ty(ty: TyType) -> Option<Self>;
    }

    #[derive(Clone)]
    pub struct Any(pub TyType);

    impl Select for Any {
        fn select_ty(ty: TyType) -> Option<Self> {
            Some(Any(ty))
        }
    }

    #[derive(Clone)]
    pub struct Named(pub NodeId, pub Vec<TyType>);

    impl Select for Named {
        fn select_ty(ty: TyType) -> Option<Self> {
            if let TyType::Named { name, parameters } = ty {
                Some(Named(name, parameters))
            } else {
                None
            }
        }
    }

    #[derive(Clone)]
    pub struct Function(pub Vec<TyType>, pub TyType);

    impl Select for Function {
        fn select_ty(ty: TyType) -> Option<Self> {
            if let TyType::Function { inputs, output } = ty {
                Some(Function(inputs, *output))
            } else {
                None
            }
        }
    }
}
