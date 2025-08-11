use derive_where::derive_where;
use std::{collections::BTreeMap, mem};

#[derive_where(Debug, Clone, PartialEq, Eq)]
pub enum Constraint<Db: crate::Db> {
    Ty(Db::Node, Ty<Db>),
    Instantiation(Instantiation<Db>),
    Bound(Bound<Db>),
}

impl<Db: crate::Db> Constraint<Db> {
    pub fn traverse_nodes_mut(&mut self, f: &mut impl FnMut(&mut Db::Node)) {
        fn traverse_node_ty<Db: crate::Db>(
            node: &mut Db::Node,
            ty: &mut Ty<Db>,
            f: &mut impl FnMut(&mut Db::Node),
        ) {
            f(node);

            ty.traverse_mut(&mut |ty| {
                if let Ty::Of(node) = ty {
                    f(node);
                }
            });
        }

        fn traverse_substitutions<Db: crate::Db>(
            substitutions: &mut Substitutions<Db>,
            f: &mut impl FnMut(&mut Db::Node),
        ) {
            substitutions.0 = mem::take(&mut substitutions.0)
                .into_iter()
                .map(|(mut node, mut ty)| {
                    traverse_node_ty(&mut node, &mut ty, f);
                    (node, ty)
                })
                .collect();
        }

        match self {
            Constraint::Ty(node, ty) => {
                traverse_node_ty(node, ty, f);
            }
            Constraint::Instantiation(instantiation) => {
                traverse_substitutions(&mut instantiation.substitutions, f);
            }
            Constraint::Bound(bound) => {
                f(&mut bound.0.node);
                traverse_substitutions(&mut bound.0.substitutions, f);
            }
        }
    }

    pub fn traverse_tys_mut(&mut self, f: &mut impl FnMut(&mut Ty<Db>)) {
        match self {
            Constraint::Ty(_, ty) => f(ty),
            Constraint::Instantiation(instantiation) => {
                for ty in instantiation.substitutions.0.values_mut() {
                    f(ty);
                }
            }
            Constraint::Bound(bound) => {
                for ty in bound.0.substitutions.0.values_mut() {
                    f(ty);
                }
            }
        }
    }
}

#[derive_where(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty<Db: crate::Db> {
    Unknown(Db::Node),
    Of(Db::Node),
    Parameter(Db::Node),
    Named {
        name: Db::Node,
        parameters: BTreeMap<Db::Node, Ty<Db>>,
    },
    Function {
        inputs: Vec<Ty<Db>>,
        output: Box<Ty<Db>>,
    },
    Tuple {
        elements: Vec<Ty<Db>>,
    },
}

impl<Db: crate::Db> Ty<Db> {
    pub fn unit() -> Self {
        Ty::Tuple {
            elements: Vec::new(),
        }
    }

    pub fn traverse(&self, f: &mut impl FnMut(&Self)) {
        f(self);

        match self {
            Ty::Unknown(_) | Ty::Of(_) | Ty::Parameter(_) => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters.values() {
                    parameter.traverse(f);
                }
            }
            Ty::Function { inputs, output } => {
                for input in inputs {
                    input.traverse(f);
                }

                output.traverse(f);
            }
            Ty::Tuple { elements } => {
                for element in elements {
                    element.traverse(f);
                }
            }
        }
    }

    pub fn traverse_mut(&mut self, f: &mut impl FnMut(&mut Self)) {
        f(self);

        match self {
            Ty::Unknown(_) | Ty::Of(_) | Ty::Parameter(_) => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters.values_mut() {
                    parameter.traverse_mut(f);
                }
            }
            Ty::Function { inputs, output } => {
                for input in inputs {
                    input.traverse_mut(f);
                }

                output.traverse_mut(f);
            }
            Ty::Tuple { elements } => {
                for element in elements {
                    element.traverse_mut(f);
                }
            }
        }
    }

    pub fn contains_node(&self, node: Db::Node) -> bool {
        let mut contains_node = false;
        self.traverse(&mut |ty| {
            if let Ty::Of(other) = ty
                && *other == node
            {
                contains_node = true;
            }
        });

        contains_node
    }

    pub fn contains_parameter(&self) -> bool {
        let mut contains_parameter = false;
        self.traverse(&mut |ty| {
            if let Ty::Parameter(_) = ty {
                contains_parameter = true;
            }
        });

        contains_parameter
    }

    pub fn is_incomplete(&self) -> bool {
        let mut incomplete = false;
        self.traverse(&mut |ty| {
            if matches!(ty, Ty::Of(_)) {
                incomplete = true;
            }
        });

        incomplete
    }
}

#[derive_where(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Substitutions<Db: crate::Db>(pub BTreeMap<Db::Node, Ty<Db>>);

impl<Db: crate::Db> Substitutions<Db> {
    pub fn replace_all() -> Self {
        Substitutions(BTreeMap::new())
    }
}

impl<Db: crate::Db> From<BTreeMap<Db::Node, Db::Node>> for Substitutions<Db> {
    fn from(value: BTreeMap<Db::Node, Db::Node>) -> Self {
        Substitutions(
            value
                .into_iter()
                .map(|(parameter, node)| (parameter, Ty::Of(node)))
                .collect(),
        )
    }
}

#[derive_where(Debug, Clone, PartialEq, Eq)]
pub struct Instantiation<Db: crate::Db> {
    pub source: Db::Node,
    pub node: Db::Node,
    pub definition: Db::Node,
    pub substitutions: Substitutions<Db>,
}

#[derive_where(Debug, Clone, PartialEq, Eq)]
pub struct Bound<Db: crate::Db>(pub Instantiation<Db>);
