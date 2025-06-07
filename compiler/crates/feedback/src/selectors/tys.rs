use crate::{Context, selectors::Select};
use wipple_compiler_trace::NodeId;
use wipple_compiler_typecheck::constraints::Ty as TyType;

#[derive(Clone)]
pub struct Ty<T: ty::Select>(pub T);

impl<T: ty::Select> Select for Ty<T> {
    fn select<'a>(ctx: &'a Context<'_>, node: NodeId, f: impl Fn(&'a Context<'_>, NodeId, Self)) {
        let Some(tys) = ctx.tys.get(&node) else {
            return;
        };

        for ty in tys {
            if let Some(ty) = T::select_ty(ty) {
                f(ctx, node, Ty(ty));
            }
        }
    }
}

#[allow(clippy::module_inception)]
pub mod ty {
    use super::*;

    pub trait Select: Clone {
        fn select_ty(ty: &TyType) -> Option<Self>;
    }

    #[derive(Clone)]
    pub struct Any(pub TyType);

    impl Select for Any {
        fn select_ty(ty: &TyType) -> Option<Self> {
            Some(Any(ty.clone()))
        }
    }

    #[derive(Clone)]
    pub struct Named(pub NodeId, pub Vec<TyType>);

    impl Select for Named {
        fn select_ty(ty: &TyType) -> Option<Self> {
            if let TyType::Named { name, parameters } = ty {
                Some(Named(*name, parameters.clone()))
            } else {
                None
            }
        }
    }

    #[derive(Clone)]
    pub struct Function(pub Vec<TyType>, pub TyType);

    impl Select for Function {
        fn select_ty(ty: &TyType) -> Option<Self> {
            if let TyType::Function { inputs, output } = ty {
                Some(Function(inputs.clone(), output.as_ref().clone()))
            } else {
                None
            }
        }
    }
}
