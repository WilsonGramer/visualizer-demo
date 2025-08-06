mod display;
mod fact;
mod node;
mod span;

pub use fact::*;
pub use node::*;
pub use span::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

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

    pub fn iter(&self, node: NodeId) -> impl Iterator<Item = &Fact> {
        self.facts
            .values()
            .filter_map(move |facts| facts.get(&node))
            .flatten()
    }

    pub fn iter_by<T: FactValue>(&self, node: NodeId, name: &str) -> impl Iterator<Item = &T> {
        self.facts
            .get(name)
            .and_then(|facts| facts.get(&node))
            .into_iter()
            .flatten()
            .filter_map(|fact| fact.value().downcast_ref::<T>())
    }

    pub fn get<T: FactValue>(&self, node: NodeId, name: &str) -> Option<&T> {
        self.facts
            .get(name)?
            .get(&node)?
            .iter()
            .find_map(|fact| fact.value().downcast_ref::<T>())
    }

    pub fn clone_node(&mut self, node: NodeId) -> NodeId {
        let new_id = self.node();

        let node_facts = self.iter(node).cloned().collect::<Vec<_>>();
        for fact in node_facts {
            self.fact(new_id, fact);
        }

        new_id
    }

    pub fn is_hidden(&self, node: NodeId) -> bool {
        self.iter(node).any(Fact::is_hidden)
    }
}
