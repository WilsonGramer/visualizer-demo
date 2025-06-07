mod queries;
mod selectors;

use crate::queries::Query;
use std::{cell::RefCell, collections::BTreeMap};
use wipple_compiler_trace::{AnyRule, NodeId, Span};
use wipple_compiler_typecheck::constraints::Ty;

#[derive(Debug, Clone, Default)]
struct Context {
    state: RefCell<State>,
    pub nodes: BTreeMap<NodeId, AnyRule>,
    pub spans: BTreeMap<NodeId, Span>,
    pub relations: BTreeMap<NodeId, (NodeId, AnyRule)>,
    pub tys: BTreeMap<NodeId, Vec<(Ty, AnyRule)>>,
}

#[derive(Debug, Clone, Default)]
struct State {
    items: Vec<FeedbackItem>,
    no_results: bool,
}

impl Context {
    fn add(&self, item: FeedbackItem) {
        self.state.borrow_mut().items.push(item);
    }

    fn run<'a, S>(&'a mut self, query: impl Query<'a, S>) {
        for node in self.nodes.keys().copied() {
            query.run(self, node);
        }
    }

    fn no_results(&self) {
        self.state.borrow_mut().no_results = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FeedbackItem {
    pub summary: Message,
    pub details: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Message {
    pub node: NodeId,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Content {
    /// Inherit the node's documentation comment
    Documentation,

    /// Provide a custom message
    Literal { segments: Vec<ContentSegment> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ContentSegment {
    Text(String),
    Link(NodeId),
}
