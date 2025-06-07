mod feedback;
mod queries;
mod selectors;

use crate::{feedback::FeedbackTemplate, queries::Query, selectors::Select};
use std::{cell::RefCell, collections::BTreeMap};
use wipple_compiler_trace::{AnyRule, NodeId, Span};
use wipple_compiler_typecheck::constraints::Ty;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    state: RefCell<State>,
    pub nodes: &'a BTreeMap<NodeId, AnyRule>,
    pub spans: &'a BTreeMap<NodeId, Span>,
    pub relations: &'a BTreeMap<NodeId, Vec<(NodeId, AnyRule)>>, // child -> parents
    pub tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
}

impl<'a> Context<'a> {
    pub fn new(
        nodes: &'a BTreeMap<NodeId, AnyRule>,
        spans: &'a BTreeMap<NodeId, Span>,
        relations: &'a BTreeMap<NodeId, Vec<(NodeId, AnyRule)>>,
        tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
    ) -> Self {
        Context {
            state: Default::default(),
            nodes,
            spans,
            relations,
            tys,
        }
    }

    pub fn collect_feedback(self) -> Vec<FeedbackTemplate> {
        queries::run(&self);
        self.state.into_inner().items
    }
}

#[derive(Debug, Clone, Default)]
struct State {
    items: Vec<FeedbackTemplate>,
}

impl Context<'_> {
    fn feedback(&self, item: FeedbackTemplate) {
        self.state.borrow_mut().items.push(item);
    }

    fn run<'a, S>(&'a self, query: impl Query<'a, S>) {
        query.run(self);
    }

    fn select_all<'a, S: Select>(&'a self, f: impl Fn(&'a Context<'_>, NodeId, S) + Copy) {
        for node in self.nodes.keys().copied() {
            S::select(self, node, f);
        }
    }
}
