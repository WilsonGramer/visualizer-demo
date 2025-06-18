use crate::{
    constraints::{Constraint, Ty},
    nodes::Node,
};

use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};
use wipple_compiler_trace::{NodeId, Rule, Span};

#[derive(Default)]
pub struct Context<'a> {
    pub nodes: BTreeMap<NodeId, (&'a dyn Node, Rule)>,
}

impl<'a> Context<'a> {
    pub fn insert(&mut self, value: &'a impl Node, rule: Rule) -> NodeId {
        let node = NodeId(self.nodes.len());
        self.nodes.insert(node, (value, rule));
        node
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = (NodeId, Rule)> {
        self.nodes.iter().map(|(&node, &(_, rule))| (node, rule))
    }

    pub(crate) fn get(&self, node: NodeId) -> (&'a dyn Node, Rule) {
        *self
            .nodes
            .get(&node)
            .unwrap_or_else(|| panic!("missing node {node:?}"))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Constant {
    pub id: NodeId,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Trait {
    pub instances: Vec<Constant>,
}

#[derive(Clone)]
pub struct FeedbackProvider<'a> {
    nodes: &'a BTreeMap<NodeId, HashSet<Rule>>,
    relations: &'a DiGraphMap<NodeId, Rule>,
    tys: &'a BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
    get_span_source: Rc<dyn Fn(NodeId) -> (Span, String) + 'a>,
}

impl<'a> FeedbackProvider<'a> {
    pub fn new(
        nodes: &'a BTreeMap<NodeId, HashSet<Rule>>,
        relations: &'a DiGraphMap<NodeId, Rule>,
        tys: &'a BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
        get_span_source: impl Fn(NodeId) -> (Span, String) + 'a,
    ) -> Self {
        FeedbackProvider {
            nodes,
            relations,
            tys,
            get_span_source: Rc::new(get_span_source),
        }
    }

    pub fn node_span_source(&self, node: NodeId) -> (Span, String) {
        (self.get_span_source)(node)
    }

    pub fn node_rules(&self, node: NodeId) -> &HashSet<Rule> {
        self.nodes.get(&node).unwrap()
    }

    pub fn node_tys(&self, node: NodeId) -> impl Iterator<Item = &Ty> {
        self.tys.get(&node).into_iter().flatten().map(|(ty, _)| ty)
    }

    pub fn related_nodes(&self, node: NodeId) -> impl Iterator<Item = (NodeId, Rule)> {
        self.relations
            .neighbors_directed(node, Direction::Incoming)
            .map(move |other| (other, *self.relations.edge_weight(other, node).unwrap()))
    }
}
