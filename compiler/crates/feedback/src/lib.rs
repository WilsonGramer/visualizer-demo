mod feedback;
mod queries;
mod selectors;

use crate::{
    feedback::{Feedback, NodeTerm, State, TermCounts, TermsIter, TyTerm},
    queries::QUERIES,
};
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use wipple_compiler_trace::{AnyRule, NodeId, Span};
use wipple_compiler_typecheck::constraints::Ty;

#[derive(Clone)]
pub struct Context<'a> {
    pub nodes: &'a BTreeMap<NodeId, AnyRule>,
    pub spans: &'a BTreeMap<NodeId, Span>,
    pub names: &'a HashMap<String, NodeId>,
    pub relations: &'a BTreeMap<NodeId, Vec<(NodeId, AnyRule)>>, // child -> parents
    pub tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
}

impl<'a> Context<'a> {
    pub fn new(
        nodes: &'a BTreeMap<NodeId, AnyRule>,
        spans: &'a BTreeMap<NodeId, Span>,
        names: &'a HashMap<String, NodeId>,
        relations: &'a BTreeMap<NodeId, Vec<(NodeId, AnyRule)>>,
        tys: &'a BTreeMap<NodeId, (Vec<Ty>, BTreeMap<NodeId, AnyRule>)>,
    ) -> Self {
        Context {
            nodes,
            spans,
            names,
            relations,
            tys,
        }
    }

    pub fn collect_feedback(&self) -> Vec<(&'static str, &'static Feedback, State)> {
        let mut result = Vec::new();
        for (name, query) in QUERIES.iter() {
            // Generate combinations of terms by counting the number of terms
            // requested in the selector (`count_terms`), and then calling
            // `Itertools::combinations` with this number.
            for (&node, &rule) in self.nodes {
                let mut term_counts = TermCounts::default();
                for selector in &query.selectors {
                    selector.count_terms(&mut term_counts);
                }

                let relation_combinations = self
                    .relations
                    .get(&node)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(parent, rule)| NodeTerm { node: parent, rule })
                    .chain([NodeTerm { node, rule }]) // include the node itself
                    .combinations(term_counts.nodes);

                let (tys, influences) = self.tys.get(&node).cloned().unwrap_or_default();

                let ty_combinations = || {
                    tys.iter().combinations(term_counts.tys).map(|tys| {
                        tys.into_iter()
                            .map(|ty| TyTerm {
                                ty: ty.clone(),
                                influences: influences
                                    .iter()
                                    .map(move |(&node, &rule)| NodeTerm { node, rule })
                                    .collect(),
                            })
                            .collect()
                    })
                };

                for relations in relation_combinations {
                    for tys in ty_combinations() {
                        // `TermsIter` creates a "cursor" for the combinations
                        // of terms...
                        let mut terms = TermsIter::new(relations.clone(), tys);

                        // ...and `State` associated the terms with keys that
                        // are substituted into the feedback message.
                        let mut state = State::new(NodeTerm { node, rule });

                        // Populate the state, relying on the same traversal
                        // order between `TermsIter` and `TermCounts`.
                        for selector in &query.selectors {
                            selector.run(&mut terms, &mut state);
                        }

                        result.push((name.as_str(), &query.item, state));
                    }
                }
            }
        }

        result
    }
}
