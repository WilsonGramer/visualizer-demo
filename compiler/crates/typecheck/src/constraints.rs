use crate::{
    context::{Context, FeedbackProvider},
    nodes::Node,
};
use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    ops::Deref,
};
use wipple_compiler_trace::{NodeId, Rule};

pub type TyConstraints = BTreeMap<NodeId, Vec<(Ty, Rule)>>;
pub type GenericConstraints = Vec<(NodeId, ((NodeId, BTreeMap<NodeId, NodeId>), Rule))>;
pub type BoundConstraints = BTreeMap<NodeId, Vec<(Bound, Rule)>>;

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub nodes: BTreeSet<NodeId>,
    pub tys: TyConstraints,
    pub generic_tys: GenericConstraints,
    pub bounds: BoundConstraints,
}

impl Constraints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_ty(&mut self, node: NodeId, ty: Ty, rule: Rule) {
        self.tys.entry(node).or_default().push((ty, rule));
    }

    pub fn insert_generic(
        &mut self,
        node: NodeId,
        definition: NodeId,
        substitutions: BTreeMap<NodeId, NodeId>,
        rule: Rule,
    ) {
        self.generic_tys
            .push((node, ((definition, substitutions), rule)));
    }

    pub fn insert_bound(&mut self, node: NodeId, bound: Bound, rule: Rule) {
        self.bounds.entry(node).or_default().push((bound, rule));
    }

    pub fn get(&self, node: &NodeId) -> impl Iterator<Item = Constraint> {
        let tys = self.tys.get(node).into_iter().flat_map(|tys| {
            tys.iter()
                .map(|(ty, rule)| Constraint::Ty(ty.clone(), *rule))
        });

        let generics = self
            .generic_tys
            .iter()
            .filter(move |(n, _)| n == node)
            .map(|(_, (bound, rule))| Constraint::Generic(bound.clone(), *rule));

        let bounds = self.bounds.get(node).into_iter().flat_map(|bounds| {
            bounds
                .iter()
                .map(|(bound, rule)| Constraint::Bound(bound.clone(), *rule))
        });

        tys.chain(generics).chain(bounds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint {
    Ty(Ty, Rule),
    Generic((NodeId, BTreeMap<NodeId, NodeId>), Rule),
    Bound(Bound, Rule),
}

impl Constraints {
    pub fn from_iter(node: NodeId, iter: impl IntoIterator<Item = Constraint>) -> Self {
        let mut constraints = Constraints::new();
        constraints.nodes.insert(node);
        constraints.extend(node, iter);
        constraints
    }

    pub fn extend(&mut self, node: NodeId, iter: impl IntoIterator<Item = Constraint>) {
        for constraint in iter {
            match constraint {
                Constraint::Ty(ty, rule) => self.insert_ty(node, ty, rule),
                Constraint::Generic((definition, substitutions), rule) => {
                    self.insert_generic(node, definition, substitutions, rule)
                }
                Constraint::Bound(bound, rule) => self.insert_bound(node, bound, rule),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, Constraint)> {
        let tys = self.tys.iter().flat_map(|(node, constraints)| {
            constraints
                .iter()
                .map(move |(ty, rule)| (*node, Constraint::Ty(ty.clone(), *rule)))
        });

        let generics = self
            .generic_tys
            .iter()
            .map(|(node, (bound, rule))| (*node, Constraint::Generic(bound.clone(), *rule)));

        let bounds = self.bounds.iter().flat_map(|(node, constraints)| {
            constraints
                .iter()
                .map(move |(bound, rule)| (*node, Constraint::Bound(bound.clone(), *rule)))
        });

        tys.chain(generics).chain(bounds)
    }
}

impl Constraint {
    pub fn traverse(&self, f: &mut impl FnMut(&Ty)) {
        match self {
            Constraint::Ty(ty, _) => ty.traverse(f),
            Constraint::Generic(_, _) => {}
            Constraint::Bound(bound, _) => {
                for parameter in &bound.parameters {
                    parameter.traverse(f);
                }
            }
        }
    }

    pub fn traverse_mut(&mut self, f: &mut impl FnMut(&mut Ty)) {
        match self {
            Constraint::Ty(ty, _) => ty.traverse_mut(f),
            Constraint::Generic(_, _) => {}
            Constraint::Bound(bound, _) => {
                for parameter in &mut bound.parameters {
                    parameter.traverse_mut(f);
                }
            }
        }
    }

    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Constraint::Ty(ty, _) => ty.to_debug_string(provider),
            Constraint::Generic((definition, _), _) => {
                format!("?(generic {})", provider.node_span_source(*definition).1)
            }
            Constraint::Bound(bound, _) => bound.to_debug_string(provider),
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
            Ty::Unknown | Ty::Of(..) => {}
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
            Ty::Unknown | Ty::Of(..) => {}
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

    pub fn is_incomplete(&self) -> bool {
        let mut incomplete = false;
        self.traverse(&mut |ty| {
            if matches!(ty, Ty::Unknown | Ty::Of(..)) {
                incomplete = true;
            }
        });

        incomplete
    }
}

impl Ty {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Ty::Unknown => String::from("_"),
            Ty::Of(node) => format!("?({})", provider.node_span_source(*node).1),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bound {
    pub tr: NodeId,
    pub parameters: Vec<Ty>,
}

impl Bound {
    fn to_debug_string(&self, _provider: &FeedbackProvider<'_>) -> String {
        String::from("(bound)")
    }
}
