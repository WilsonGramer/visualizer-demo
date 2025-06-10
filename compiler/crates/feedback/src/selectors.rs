use crate::feedback::{State, TermCounts, TermsIter};
use schemars::JsonSchema;
use serde::Deserialize;
use wipple_compiler_trace::Rule;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    And(Vec<Selector>),
    Node {
        r#as: String,
        rule: Option<String>,
    },
    Child {
        r#as: String,
        parent: String,
        rule: Option<String>,
    },
    Type {
        r#as: String,
        rule: Option<String>,
    },
}

impl Selector {
    pub fn count_terms(&self, term_counts: &mut TermCounts) {
        match self {
            Selector::And(selectors) => {
                for selector in selectors {
                    selector.count_terms(term_counts);
                }
            }
            Selector::Node { .. } => {
                // Do nothing; the node is included in the state directly
            }
            Selector::Child { .. } => {
                term_counts.nodes += 2;
            }
            Selector::Type { .. } => {
                term_counts.tys += 1;
            }
        }
    }

    pub fn run(&self, terms: &mut TermsIter, state: &mut State) {
        match self {
            Selector::And(selectors) => {
                for selector in selectors {
                    selector.run(terms, state);
                }
            }
            Selector::Node { r#as: name, rule } => {
                if rule
                    .as_ref()
                    .is_none_or(|rule| state.node.rule.name() == rule)
                {
                    state.nodes.insert(name.clone(), state.node.clone());
                }
            }
            Selector::Child {
                r#as: name,
                parent: parent_name,
                rule,
            } => {
                let term = terms.relation(name);

                if rule.as_ref().is_none_or(|rule| term.rule.name() == rule) {
                    state.nodes.insert(name.clone(), state.node.clone());
                    state.nodes.insert(parent_name.clone(), term);
                }
            }
            Selector::Type { r#as: name, rule } => {
                let term = terms.ty(name);

                if term.influences.is_empty()
                    || term.influences.iter().any(|influence| {
                        rule.as_ref()
                            .is_none_or(|rule| influence.rule.name() == rule)
                    })
                {
                    state.tys.insert(name.clone(), term.clone());
                }
            }
        }
    }
}
