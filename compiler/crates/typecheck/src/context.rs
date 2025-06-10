use crate::{
    constraints::{Constraint, Ty},
    nodes::Node,
};

use std::{collections::BTreeMap, rc::Rc};
use wipple_compiler_trace::{AnyRule, NodeId, Span};

#[derive(Default)]
pub struct Context<'a> {
    pub nodes: BTreeMap<NodeId, &'a dyn Node>,
}

impl<'a> Context<'a> {
    pub fn insert(&mut self, value: &'a impl Node) -> NodeId {
        let node = NodeId(self.nodes.len());
        self.nodes.insert(node, value);
        node
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.keys().copied()
    }

    pub(crate) fn get(&self, node: NodeId) -> &'a dyn Node {
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
    tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
    get_span_source: Rc<dyn Fn(NodeId) -> (Span, String) + 'a>,
}

impl<'a> FeedbackProvider<'a> {
    pub fn new(
        tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
        get_span_source: impl Fn(NodeId) -> (Span, String) + 'a,
    ) -> Self {
        FeedbackProvider {
            tys,
            get_span_source: Rc::new(get_span_source),
        }
    }

    pub fn node_span_source(&self, node: NodeId) -> (Span, String) {
        (self.get_span_source)(node)
    }

    pub fn node_tys(&self, node: NodeId) -> impl Iterator<Item = String> {
        self.tys
            .get(&node)
            .into_iter()
            .flat_map(|(tys, _)| tys)
            .map(|ty| ty.to_debug_string(self))
    }

    pub fn related_nodes(&self, node: NodeId) -> impl Iterator<Item = (NodeId, AnyRule)> {
        self.tys
            .get(&node)
            .into_iter()
            .flat_map(|(_, related)| related.iter().map(|(&id, &rule)| (id, rule)))
    }
}
