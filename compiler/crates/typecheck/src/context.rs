use crate::{constraints::Constraint, nodes::Node};
use std::{collections::BTreeMap, rc::Rc};
use wipple_compiler_trace::{NodeId, Span};

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
pub struct DebugProvider<'a> {
    pub debug: Rc<dyn Fn(NodeId, DebugOptions) -> (Span, String) + 'a>,
}

#[derive(Default)]
#[non_exhaustive]
pub struct DebugOptions {
    pub rule: bool,
}

impl<'a> DebugProvider<'a> {
    pub fn new(debug: impl Fn(NodeId, DebugOptions) -> (Span, String) + 'a) -> Self {
        DebugProvider {
            debug: Rc::new(debug),
        }
    }

    pub fn node(&self, node: NodeId, options: DebugOptions) -> (Span, String) {
        (self.debug)(node, options)
    }
}
