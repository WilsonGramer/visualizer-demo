use crate::{constraints::Constraint, id::TypedNodeId, nodes::Node};
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};
use wipple_compiler_trace::{NodeId, Rule, Span};

#[derive(Debug, Default)]
pub struct Context<'a> {
    pub nodes: BTreeMap<NodeId, (&'a dyn Node, Rule)>,
}

impl<'a> Context<'a> {
    pub fn insert(&mut self, value: &'a impl Node, rule: Rule) -> NodeId {
        let node = NodeId::new(self.nodes.len() as u32);
        self.nodes.insert(node, (value, rule));
        node
    }

    pub(crate) fn get(&self, node: NodeId) -> (&'a dyn Node, Rule) {
        *self
            .nodes
            .get(&node)
            .unwrap_or_else(|| panic!("missing node {node:?}"))
    }
}

#[derive(Clone)]
pub struct Constant {
    pub id: NodeId,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone)]
pub struct Trait {
    pub instances: Vec<Constant>,
}

#[derive(Clone)]
pub struct FeedbackProvider<'a> {
    nodes: &'a BTreeMap<TypedNodeId, HashSet<Rule>>,
    relations: &'a DiGraphMap<NodeId, Rule>,
    get_span_source: Rc<dyn Fn(NodeId) -> (Span, String) + 'a>,
    get_comments: Rc<dyn Fn(NodeId) -> Option<String> + 'a>,
}

impl<'a> FeedbackProvider<'a> {
    pub fn new(
        nodes: &'a BTreeMap<TypedNodeId, HashSet<Rule>>,
        relations: &'a DiGraphMap<NodeId, Rule>,
        get_span_source: impl Fn(NodeId) -> (Span, String) + 'a,
        get_comments: impl Fn(NodeId) -> Option<String> + 'a,
    ) -> Self {
        FeedbackProvider {
            nodes,
            relations,
            get_span_source: Rc::new(get_span_source),
            get_comments: Rc::new(get_comments),
        }
    }

    pub fn related_nodes(&self, node: NodeId) -> impl Iterator<Item = (NodeId, Rule)> {
        self.relations
            .neighbors_directed(node, Direction::Incoming)
            .map(move |other| (other, *self.relations.edge_weight(other, node).unwrap()))
    }

    pub fn node_span_source(&self, node: NodeId) -> (Span, String) {
        (self.get_span_source)(node)
    }

    pub fn comments(&self, node: NodeId) -> Option<String> {
        (self.get_comments)(node)
    }

    pub fn node_rules(&self, node: TypedNodeId) -> &HashSet<Rule> {
        self.nodes.get(&node).unwrap()
    }
}
