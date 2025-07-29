use crate::{feedback::FeedbackProvider, nodes::Node};
use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};
use wipple_compiler_trace::{NodeId, Rule};

pub type TyConstraints = BTreeMap<NodeId, Vec<(Ty, Rule)>>;
pub type InstantiationConstraints = Vec<Instantiation>;
pub type BoundConstraints = Vec<(Bound, Rule)>;

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub nodes: BTreeSet<NodeId>,
    pub tys: TyConstraints,
    pub instantiations: InstantiationConstraints,
    pub bounds: BoundConstraints,
}

impl Constraints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_ty(&mut self, node: NodeId, ty: Ty, rule: Rule) {
        self.tys.entry(node).or_default().push((ty, rule));
    }

    pub fn insert_instantiation(&mut self, instantiation: Instantiation) {
        self.instantiations.push(instantiation);
    }

    pub fn insert_bound(&mut self, bound: Bound, rule: Rule) {
        self.bounds.push((bound, rule));
    }
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Ty(NodeId, Ty, Rule),
    Instantiation(Instantiation),
    Bound(Bound, Rule),
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
                Constraint::Instantiation(instantiation) => {
                    self.insert_instantiation(instantiation)
                }
                Constraint::Bound(bound, rule) => self.insert_bound(bound, rule),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Constraint> {
        let tys = self.tys.iter().flat_map(|(node, constraints)| {
            constraints
                .iter()
                .map(move |(ty, rule)| (Constraint::Ty(*node, ty.clone(), *rule)))
        });

        let instantiations = self
            .instantiations
            .iter()
            .map(|instantiation| Constraint::Instantiation(instantiation.clone()));

        let bounds = self
            .bounds
            .iter()
            .map(|(bound, rule)| Constraint::Bound(bound.clone(), *rule));

        tys.chain(instantiations).chain(bounds)
    }
}

impl Constraint {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Constraint::Ty(_, ty, _) => ty.to_debug_string(provider),
            Constraint::Instantiation(..) | Constraint::Bound(..) => {
                String::from("(instantiation)")
            }
        }
    }
}

pub trait ToConstraints {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>);
}

#[derive(Default)]
pub struct ToConstraintsContext<'a> {
    constraints: RefCell<Constraints>,
    rules: BTreeMap<TypeId, Box<dyn Fn(NodeId, &dyn Node, &Self)>>,
}

impl<'a> ToConstraintsContext<'a> {
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

    pub fn visit(&mut self, id: NodeId, node: &dyn Node) {
        let Some(rule) = self.rules.get(&node.type_id()) else {
            return;
        };

        rule(id, node, self);
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
    Parameter(NodeId),
    Named {
        name: NodeId,
        parameters: BTreeMap<NodeId, Ty>,
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
    pub fn unit() -> Self {
        Ty::Tuple {
            elements: Vec::new(),
        }
    }

    pub fn traverse(&self, f: &mut impl FnMut(&Self)) {
        f(self);

        match self {
            Ty::Unknown | Ty::Of(_) | Ty::Parameter(_) => {}
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
            Ty::Unknown | Ty::Of(_) | Ty::Parameter(_) => {}
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

impl Ty {
    pub fn to_debug_string(&self, provider: &FeedbackProvider<'_>) -> String {
        match self {
            Ty::Unknown | Ty::Of(_) => String::from("_"),
            Ty::Parameter(node) => provider.node_span_source(*node).1,
            Ty::Named { name, parameters } => format!(
                "{}{}",
                provider.node_span_source(*name).1,
                parameters
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
pub struct Substitutions(pub BTreeMap<NodeId, Ty>);

impl Substitutions {
    pub fn replace_all() -> Self {
        Substitutions(BTreeMap::new())
    }
}

impl From<BTreeMap<NodeId, NodeId>> for Substitutions {
    fn from(value: BTreeMap<NodeId, NodeId>) -> Self {
        Substitutions(
            value
                .into_iter()
                .map(|(parameter, node)| (parameter, Ty::Of(node)))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Instantiation {
    pub substitutions: Substitutions,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Bound {
    pub node: NodeId,
    pub tr: NodeId,
    pub substitutions: Substitutions,
}
