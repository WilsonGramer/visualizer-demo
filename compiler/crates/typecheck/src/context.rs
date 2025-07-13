use crate::{constraints::Constraint, nodes::Node};
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
        let node = NodeId {
            namespace: None,
            index: self.nodes.len() as u32,
        };

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
    nodes: &'a BTreeMap<NodeId, HashSet<Rule>>,
    replacements: &'a BTreeMap<NodeId, NodeId>,
    relations: &'a DiGraphMap<NodeId, Rule>,
    get_span_source: Rc<dyn Fn(NodeId) -> (Span, String) + 'a>,
    get_comments: Rc<dyn Fn(NodeId) -> Option<String> + 'a>,
}

impl<'a> FeedbackProvider<'a> {
    pub fn new(
        nodes: &'a BTreeMap<NodeId, HashSet<Rule>>,
        replacements: &'a BTreeMap<NodeId, NodeId>,
        relations: &'a DiGraphMap<NodeId, Rule>,
        get_span_source: impl Fn(NodeId) -> (Span, String) + 'a,
        get_comments: impl Fn(NodeId) -> Option<String> + 'a,
    ) -> Self {
        FeedbackProvider {
            nodes,
            replacements,
            relations,
            get_span_source: Rc::new(get_span_source),
            get_comments: Rc::new(get_comments),
        }
    }

    pub fn node_span_source(&self, node: NodeId) -> (Span, String) {
        (self.get_span_source)(node)
    }

    pub fn comments(&self, node: NodeId) -> Option<String> {
        (self.get_comments)(node)
    }

    pub fn node_rules(&self, node: NodeId) -> &HashSet<Rule> {
        self.nodes.get(&node).unwrap()
    }

    pub fn replacement_node(&self, node: NodeId) -> Option<NodeId> {
        self.replacements.get(&node).copied()
    }

    pub fn related_nodes(&self, node: NodeId) -> impl Iterator<Item = (NodeId, Rule)> {
        self.relations
            .neighbors_directed(node, Direction::Incoming)
            .map(move |other| (other, *self.relations.edge_weight(other, node).unwrap()))
    }
}
