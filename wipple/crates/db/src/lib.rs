mod constraints;
mod fact;
mod node;
mod query;
mod span;
mod write;

pub use constraints::*;
pub use fact::*;
pub use node::*;
pub use query::*;
pub use span::*;
pub use write::*;

use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};
use visualizer::{Constraint, Instantiation, Substitutions, Ty};

#[derive(Debug, Clone, Default)]
pub struct Db {
    next_id: u32,
    facts: HashMap<Rc<str>, BTreeMap<NodeId, Vec<Fact>>>,
}

impl Db {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn node(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn fact(&mut self, node: NodeId, fact: Fact) {
        self.facts
            .entry(fact.name.clone())
            .or_default()
            .entry(node)
            .or_default()
            .push(fact);
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.facts
            .values()
            .flat_map(|facts| facts.keys())
            .cloned()
            .collect::<BTreeSet<_>>() // deduplicate
            .into_iter()
    }

    pub fn all(&self, name: &str) -> impl Iterator<Item = (NodeId, &Fact)> {
        self.facts
            .get(name)
            .into_iter()
            .flatten()
            .flat_map(|(&node, facts)| facts.iter().map(move |fact| (node, fact)))
    }

    pub fn iter(&self, node: NodeId) -> impl Iterator<Item = &Fact> {
        self.facts
            .values()
            .filter_map(move |facts| facts.get(&node))
            .flatten()
    }

    pub fn iter_by(&self, node: NodeId, name: &str) -> impl Iterator<Item = &Fact> {
        self.facts
            .get(name)
            .and_then(|facts| facts.get(&node))
            .into_iter()
            .flatten()
    }

    pub fn iter_of<T: FactValue>(&self, node: NodeId, name: &str) -> impl Iterator<Item = &T> {
        self.iter_by(node, name)
            .filter_map(|fact| fact.value().downcast_ref::<T>())
    }

    pub fn get<T: FactValue>(&self, node: NodeId, name: &str) -> Option<&T> {
        self.facts
            .get(name)?
            .get(&node)?
            .iter()
            .find_map(|fact| fact.value().downcast_ref::<T>())
    }

    pub fn clone_node_tree_inner(
        &mut self,
        node: NodeId,
        copy: NodeId,
        substitutions: &mut Substitutions<Self>,
        hide: bool,
        copies: &mut BTreeMap<NodeId, NodeId>,
        constraints: &mut Vec<Constraint<Self>>,
    ) {
        copies.insert(node, copy);

        if hide {
            self.fact(copy, Fact::hidden());
        }

        let (node_constraints, node_facts): (Vec<_>, Vec<_>) =
            self.iter(node).cloned().partition_map(|fact| {
                if let Some(constraint) = fact.value().downcast_ref::<LazyConstraints>() {
                    itertools::Either::Left(constraint.clone())
                } else {
                    itertools::Either::Right(fact)
                }
            });

        for fact in node_facts {
            self.fact(copy, fact);
        }

        self.fact(copy, Fact::new("instantiated", ()));

        for mut constraint in node_constraints
            .into_iter()
            .flat_map(|constraints| constraints.resolve_for(copy))
            .collect::<Vec<_>>()
        {
            constraint.traverse_tys_mut(&mut |ty| {
                if let Ty::Parameter(parameter) = *ty {
                    if let Some(substitution) = substitutions.0.get(&parameter).cloned() {
                        *ty = substitution;
                    } else {
                        let copy = self.node();
                        substitutions.0.insert(parameter, Ty::Of(copy));
                        *ty = Ty::Of(copy);

                        self.clone_node_tree_inner(
                            parameter,
                            copy,
                            substitutions,
                            false,
                            copies,
                            constraints,
                        );
                    }
                }
            });

            constraint.traverse_nodes_mut(&mut |node| {
                if let Some(copy) = copies.get(node) {
                    *node = *copy;
                } else {
                    let copy = self.node();

                    self.clone_node_tree_inner(
                        *node,
                        copy,
                        substitutions,
                        false,
                        copies,
                        constraints,
                    );

                    *node = copy;
                }
            });

            constraints.push(constraint.clone());
        }
    }

    pub fn is_hidden(&self, node: NodeId) -> bool {
        self.iter(node).any(Fact::is_hidden)
    }
}

impl visualizer::Db for Db {
    type Node = NodeId;

    fn typed_nodes(&self) -> impl Iterator<Item = Self::Node> {
        self.nodes()
            .filter(|&node| !self.is_hidden(node) && self.get::<()>(node, "untyped").is_none())
    }

    fn clone_node_tree(
        &mut self,
        node: Self::Node,
        substitutions: &mut Substitutions<Self>,
        hide: bool,
    ) -> (Self::Node, Vec<Constraint<Self>>) {
        let copy = self.node();

        let mut constraints = Vec::new();
        self.clone_node_tree_inner(
            node,
            copy,
            substitutions,
            hide,
            &mut BTreeMap::new(),
            &mut constraints,
        );

        (copy, constraints)
    }

    fn get_trait_instances(
        &mut self,
        source: Self::Node,
        node: Self::Node,
        trait_id: Self::Node,
    ) -> Vec<(Self::Node, Instantiation<Self>)> {
        self.iter_of(trait_id, "instance")
            .map(|&instance| {
                let substitutions = self
                    .get::<Substitutions<Self>>(instance, "substitutions")
                    .unwrap()
                    .clone();

                let instantiation = Instantiation {
                    source,
                    node,
                    definition: trait_id,
                    substitutions,
                };

                (instance, instantiation)
            })
            .collect()
    }

    fn flag_resolved(&mut self, node: Self::Node, instance: Self::Node, ty: NodeId) {
        self.fact(node, Fact::new("resolvedTrait", ty));
        self.fact(ty, Fact::new("resolvedInstance", instance));
    }

    fn flag_unresolved(&mut self, node: Self::Node, ty: NodeId) {
        self.fact(node, Fact::new("unresolvedTrait", ty));
    }

    fn flag_type(&mut self, node: Self::Node, ty: Ty<Self>) {
        self.fact(node, Fact::new("type", ty));
    }

    fn flag_incomplete_type(&mut self, node: Self::Node) {
        self.fact(node, Fact::new("incompleteType", ()));
    }
}
