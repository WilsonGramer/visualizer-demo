use crate::{
    constraints::Constraint,
    util::{Fact, NodeId, Span},
};
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

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
    facts: &'a BTreeMap<NodeId, HashSet<Fact>>,
    get_span_source: Rc<dyn Fn(NodeId) -> (Span, String) + 'a>,
    get_comments: Rc<dyn Fn(NodeId) -> Option<String> + 'a>,
}

impl<'a> FeedbackProvider<'a> {
    pub fn new(
        facts: &'a BTreeMap<NodeId, HashSet<Fact>>,
        get_span_source: impl Fn(NodeId) -> (Span, String) + 'a,
        get_comments: impl Fn(NodeId) -> Option<String> + 'a,
    ) -> Self {
        FeedbackProvider {
            facts,
            get_span_source: Rc::new(get_span_source),
            get_comments: Rc::new(get_comments),
        }
    }

    pub fn node_facts(&self, node: NodeId) -> &HashSet<Fact> {
        self.facts.get(&node).unwrap()
    }

    pub fn node_span_source(&self, node: NodeId) -> (Span, String) {
        (self.get_span_source)(node)
    }

    pub fn node_comments(&self, node: NodeId) -> Option<String> {
        (self.get_comments)(node)
    }
}
