use crate::{
    context::{Context, FeedbackProvider},
    id::TypedNodeId,
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

pub type TyConstraints = BTreeMap<TypedNodeId, Vec<(Ty, Rule)>>;
pub type BoundConstraints = BTreeMap<NodeId, Vec<(Bound, Rule)>>;

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub nodes: BTreeSet<NodeId>,
    pub tys: TyConstraints,
    pub bounds: BoundConstraints,
}

impl Constraints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_ty(&mut self, node: impl Into<TypedNodeId>, ty: Ty, rule: Rule) {
        self.tys.entry(node.into()).or_default().push((ty, rule));
    }

    pub fn insert_bound(&mut self, node: NodeId, bound: Bound, rule: Rule) {
        self.bounds.entry(node).or_default().push((bound, rule));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint {
    Ty(TypedNodeId, Ty, Rule),
    Bound(NodeId, Bound, Rule),
}

impl Constraints {
    pub fn from_iter(node: NodeId, iter: impl IntoIterator<Item = Constraint>) -> Self {
        let mut constraints = Constraints::new();
        constraints.nodes.insert(node);
        constraints.extend(iter);
        constraints
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = Constraint>) {
        for constraint in iter {
            match constraint {
                Constraint::Ty(node, ty, rule) => self.insert_ty(node, ty, rule),
                Constraint::Bound(node, bound, rule) => self.insert_bound(node, bound, rule),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Constraint> {
        let tys = self.tys.iter().flat_map(|(node, constraints)| {
            constraints
                .iter()
                .map(move |(ty, rule)| (Constraint::Ty(*node, ty.clone(), *rule)))
        });

        let bounds = self.bounds.iter().flat_map(|(node, constraints)| {
            constraints
                .iter()
                .map(move |(bound, rule)| (Constraint::Bound(*node, bound.clone(), *rule)))
        });

        tys.chain(bounds)
    }
}

impl Constraint {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Constraint::Ty(_, ty, _) => ty.to_debug_string(provider),
            Constraint::Bound(_, bound, _) => bound.to_debug_string(provider),
        }
    }
}

pub trait ToConstraints {
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

    pub fn register<N: Node + ToConstraints>(&mut self) {
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
    Of(TypedNodeId),
    Parameter(NodeId),
    Instantiated(NodeId),
    Instantiate {
        instantiation: NodeId,
        definition: NodeId,
        // `None` is equivalent to substituting all parameters
        substitutions: Option<BTreeMap<NodeId, NodeId>>,
    },
    Named {
        name: NodeId,
        substitutions: BTreeMap<NodeId, Ty>,
    },
    Function {
        inputs: Vec<Ty>,
        output: Box<Ty>,
    },
    Tuple {
        elements: Vec<Ty>,
    },
}

impl Ty {
    pub fn of(node: impl Into<TypedNodeId>) -> Self {
        Ty::Of(node.into())
    }

    pub fn unit() -> Self {
        Ty::Tuple {
            elements: Vec::new(),
        }
    }

    pub fn traverse(&self, f: &mut impl FnMut(&Self)) {
        f(self);

        match self {
            Ty::Unknown
            | Ty::Of(..)
            | Ty::Parameter(..)
            | Ty::Instantiated(..)
            | Ty::Instantiate { .. } => {}
            Ty::Named { substitutions, .. } => {
                for parameter in substitutions.values() {
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
            Ty::Unknown
            | Ty::Of(..)
            | Ty::Parameter(..)
            | Ty::Instantiated(..)
            | Ty::Instantiate { .. } => {}
            Ty::Named { substitutions, .. } => {
                for parameter in substitutions.values_mut() {
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
            if matches!(ty, Ty::Of(..)) {
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
            Ty::Of(node) => format!("{node:?}"),
            Ty::Parameter(parameter) => provider.node_span_source(*parameter).1,
            Ty::Instantiated(parameter) => {
                format!(
                    "?(instantiated {})",
                    provider.node_span_source(*parameter).1
                )
            }
            Ty::Instantiate { definition, .. } => {
                format!(
                    "?(instantiate {})",
                    provider.node_span_source(*definition).1
                )
            }
            Ty::Named {
                name,
                substitutions,
            } => format!(
                "{}{}",
                provider.node_span_source(*name).1,
                substitutions
                    .values()
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
    pub instantiation: NodeId,
    pub tr: NodeId,
    // `None` is equivalent to substituting all parameters
    pub substitutions: Option<BTreeMap<NodeId, NodeId>>,
}

impl Bound {
    pub fn to_debug_string(&self, _provider: &FeedbackProvider<'_>) -> String {
        String::from("(bound)")
    }

    pub fn as_ty(&self) -> Ty {
        Ty::Instantiate {
            instantiation: self.instantiation,
            definition: self.tr,
            substitutions: self.substitutions.clone(),
        }
    }
}
