mod feedback;
mod queries;
mod selectors;

use crate::{
    feedback::Feedback,
    queries::QUERIES,
    selectors::{NodeTerm, State},
};
use petgraph::prelude::DiGraphMap;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    mem,
};
use wipple_compiler_trace::{AnyRule, NodeId, Rule, Span};
use wipple_compiler_typecheck::{constraints::Ty, context::FeedbackProvider};

#[derive(Clone)]
pub struct Context<'a> {
    pub feedback: &'a FeedbackProvider<'a>,
    pub nodes: &'a BTreeMap<NodeId, HashSet<AnyRule>>,
    pub spans: &'a BTreeMap<NodeId, Span>,
    pub names: &'a HashMap<String, NodeId>,
    pub relations: &'a DiGraphMap<NodeId, AnyRule>,
    pub groups: &'a BTreeMap<usize, BTreeMap<NodeId, Option<AnyRule>>>,
    pub tys: &'a BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
}

impl<'a> Context<'a> {
    pub fn new(
        feedback: &'a FeedbackProvider<'a>,
        nodes: &'a BTreeMap<NodeId, HashSet<AnyRule>>,
        spans: &'a BTreeMap<NodeId, Span>,
        names: &'a HashMap<String, NodeId>,
        relations: &'a DiGraphMap<NodeId, AnyRule>,
        groups: &'a BTreeMap<usize, BTreeMap<NodeId, Option<AnyRule>>>,
        tys: &'a BTreeMap<NodeId, Vec<(Ty, Option<usize>)>>,
    ) -> Self {
        Context {
            feedback,
            nodes,
            spans,
            names,
            relations,
            groups,
            tys,
        }
    }

    pub fn collect_feedback(
        &self,
    ) -> Vec<(&'static str, NodeId, &'static Feedback, State<'a, '_>)> {
        let mut result = Vec::new();

        for (name, query) in QUERIES.iter() {
            // Try every starting node
            for (&node, rules) in self.nodes {
                for &rule in rules {
                    if query
                        .rule
                        .as_deref()
                        .is_some_and(|query_rule| rule.name() != query_rule)
                    {
                        continue;
                    }

                    let term = NodeTerm { node, rule };

                    let mut state = State::new(self, query.r#as.clone(), term);

                    let next = 'outer: loop {
                        let mut next_state = state.clone();
                        for selector in &query.selectors {
                            if selector.run(&mut next_state).is_err() {
                                break 'outer None;
                            }
                        }

                        let node_progress = mem::take(&mut next_state.node_progress);
                        let ty_progress = mem::take(&mut next_state.ty_progress);

                        if node_progress {
                            if !ty_progress {
                                next_state.visited_tys = HashSet::new();
                            }

                            state = next_state;
                        } else {
                            break Some((name.as_str(), node, &query.item, next_state));
                        }
                    };

                    if let Some(next) = next {
                        result.push(next);
                    }
                }
            }
        }

        result.sort_by_key(|(_, node, _, _)| {
            let span = self.spans.get(node).unwrap();

            // Show smaller spans first
            (span.range.len(), span.range.start)
        });

        result
    }
}
