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

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};
use visualizer::{Instantiation, Substitutions, Ty};

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

    pub fn clone_node(&mut self, node: NodeId, hide: bool) -> NodeId {
        let new_id = self.node();

        if hide {
            self.fact(new_id, Fact::hidden());
        }

        let node_facts = self
            .iter(node)
            .filter(|fact| fact.name() != "untyped")
            .cloned()
            .collect::<Vec<_>>();

        for fact in node_facts {
            self.fact(new_id, fact);
        }

        new_id
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

    fn clone_node(&mut self, node: Self::Node, hide: bool) -> Self::Node {
        self.clone_node(node, hide)
    }

    fn get_trait_instances(
        &mut self,
        source: Self::Node,
        node: Self::Node,
        trait_id: Self::Node,
    ) -> Vec<(Self::Node, Instantiation<Self>)> {
        self.iter_of(trait_id, "instance")
            .map(|&instance| {
                let constraints = self
                    .get::<LazyConstraints>(instance, "constraints")
                    .unwrap()
                    .clone();

                let substitutions = self
                    .get::<Substitutions<Self>>(instance, "substitutions")
                    .unwrap()
                    .clone();

                let instantiation = Instantiation {
                    source,
                    constraints: constraints.resolve_for(node),
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
