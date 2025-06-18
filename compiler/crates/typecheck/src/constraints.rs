use crate::{
    context::{Context, FeedbackProvider},
    nodes::Node,
};
use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    fmt::Debug,
    ops::Deref,
};
use wipple_compiler_trace::NodeId;

pub type TyConstraints = BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>;
pub type BoundConstraints = BTreeMap<NodeId, Vec<Bound>>;

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub tys: TyConstraints,
    pub bounds: BoundConstraints,
}

impl Constraints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_ty(&mut self, node: NodeId, ty: Ty) {
        self.tys.entry(node).or_default().push((ty, None));
    }

    pub fn insert_bound(&mut self, node: NodeId, bound: Bound) {
        self.bounds.entry(node).or_default().push(bound);
    }

    pub fn get(&self, node: &NodeId) -> impl Iterator<Item = Constraint> {
        let tys = self
            .tys
            .get(node)
            .into_iter()
            .flat_map(|tys| tys.iter().map(|(ty, _)| Constraint::Ty(ty.clone())));

        let bounds = self
            .bounds
            .get(node)
            .into_iter()
            .flat_map(|bounds| bounds.iter().map(|bound| Constraint::Bound(bound.clone())));

        tys.chain(bounds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint {
    Ty(Ty),
    Bound(Bound),
}

impl Constraints {
    pub fn extend(&mut self, node: NodeId, iter: impl IntoIterator<Item = Constraint>) {
        for constraint in iter {
            match constraint {
                Constraint::Ty(ty) => self.insert_ty(node, ty),
                Constraint::Bound(bound) => self.insert_bound(node, bound),
            }
        }
    }
}

impl Constraint {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Constraint::Ty(ty) => ty.to_debug_string(provider),
            Constraint::Bound(bound) => bound.to_debug_string(provider),
        }
    }
}

pub trait ToConstraints: Node + Sized {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>);
}

pub struct ToConstraintsContext<'a> {
    ctx: &'a Context<'a>,
    constraints: RefCell<Constraints>,
    rules: BTreeMap<TypeId, Box<dyn Fn(NodeId, &dyn Node, &Self)>>,
}

impl<'a> Deref for ToConstraintsContext<'a> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a> ToConstraintsContext<'a> {
    pub fn new(ctx: &'a Context<'a>) -> Self {
        ToConstraintsContext {
            ctx,
            constraints: Default::default(),
            rules: Default::default(),
        }
    }

    pub fn register<N: ToConstraints>(&mut self) {
        let type_id = TypeId::of::<N>();

        self.rules.insert(
            type_id,
            Box::new(move |node_id, node, ctx| {
                let Some(node) = node.downcast::<N>() else {
                    return;
                };

                node.to_constraints(node_id, ctx);
            }),
        );
    }

    pub fn visit(&mut self, node_id: NodeId) {
        let (node, _) = self.ctx.get(node_id);

        let Some(rule) = self.rules.get(&node.type_id()) else {
            return;
        };

        rule(node_id, node, self);
    }

    pub fn into_constraints(self) -> Constraints {
        self.constraints.into_inner()
    }

    pub fn constraints(&self) -> RefMut<'_, Constraints> {
        self.constraints.borrow_mut()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Unknown,
    Of(NodeId),
    Instantiate(NodeId),
    Group(usize),
    Named { name: NodeId, parameters: Vec<Ty> },
    Function { inputs: Vec<Ty>, output: Box<Ty> },
    Tuple { elements: Vec<Ty> },
}

impl Ty {
    pub fn unit() -> Self {
        Ty::Tuple {
            elements: Vec::new(),
        }
    }

    pub fn traverse(&self, f: &mut impl FnMut(&Self)) {
        f(self);

        match self {
            Ty::Group(_) | Ty::Unknown | Ty::Of(..) | Ty::Instantiate { .. } => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters {
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
            Ty::Group(_) | Ty::Unknown | Ty::Of(..) | Ty::Instantiate { .. } => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters {
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

    pub fn needs_instantiation(&self) -> bool {
        let mut needs_instantiation = false;
        self.traverse(&mut |ty| {
            if matches!(ty, Ty::Instantiate(_)) {
                needs_instantiation = true;
            }
        });

        needs_instantiation
    }

    pub fn is_incomplete(&self) -> bool {
        let mut incomplete = false;
        self.traverse(&mut |ty| {
            if matches!(
                ty,
                Ty::Group(_) | Ty::Unknown | Ty::Of(..) | Ty::Instantiate { .. }
            ) {
                incomplete = true;
            }
        });

        incomplete
    }

    pub fn relative_ordering(&self) -> usize {
        match self {
            Ty::Instantiate { .. } => 0,
            Ty::Unknown | Ty::Of(..) | Ty::Group(_) => 1,
            _ => 2,
        }
    }
}

impl Ty {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Ty::Group(var) => format!("({var})"),
            Ty::Unknown => String::from("_"),
            Ty::Of(node) | Ty::Instantiate(node) => {
                format!("({})", provider.node_span_source(*node).1)
            }
            Ty::Named { name, parameters } => format!(
                "{}{}",
                provider.node_span_source(*name).1,
                parameters
                    .iter()
                    .map(|parameter| format!(" {}", parameter.to_debug_string(provider)))
                    .collect::<String>()
            ),
            Ty::Function { inputs, output } => format!(
                "{}-> {}",
                inputs
                    .iter()
                    .map(|input| format!("{} ", input.to_debug_string(provider)))
                    .collect::<String>(),
                output.to_debug_string(provider)
            ),
            Ty::Tuple { elements } => format!(
                "({})",
                elements
                    .iter()
                    .map(|element| element.to_debug_string(provider))
                    .collect::<Vec<_>>()
                    .join(" ; ")
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bound {
    pub source: NodeId,
    pub r#trait: NodeId,
    pub parameters: Vec<Ty>,
}

impl Bound {
    fn to_debug_string(&self, _provider: &FeedbackProvider<'_>) -> String {
        String::from("(bound)")
    }
}
