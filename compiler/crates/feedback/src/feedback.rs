use crate::selectors::State;
use itertools::Itertools;
use petgraph::{Direction, graphmap::NodeTrait, prelude::DiGraphMap};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    sync::LazyLock,
};
use wipple_compiler_typecheck::context::FeedbackProvider;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Feedback {
    pub group: FeedbackGroup,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackGroup {
    Todo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Message {
    Node {
        node: String,

        #[serde(flatten)]
        content: Content,
    },

    Trace {
        trace: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct MessageOptions {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Content {
    /// Inherit a node's documentation comment
    Documentation(String),

    /// Provide a message from a template
    Template(String),
}

fn render_template(
    template: &str,
    state: &State<'_, '_>,
    provider: &FeedbackProvider<'_>,
) -> Option<String> {
    static TEMPLATE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{\{([^\}]+)\}\}").unwrap());

    let mut valid = true;
    let replacement = TEMPLATE_REGEX.replace_all(template, |captures: &regex::Captures<'_>| {
        let capture = captures.get(1).unwrap();
        let (name, display): (&str, &str) = capture
            .as_str()
            .split(':')
            .collect_tuple()
            .unwrap_or_else(|| panic!("invalid interpolation `{}`", capture.as_str()));

        let replacement = (|| {
            match display {
                "source" => {
                    let term = state.nodes.get(name)?;

                    let (span, source) = provider.node_span_source(term.node);

                    // TODO: Generate link using the span
                    let _ = span;

                    Some(source)
                }
                "type" => {
                    let term = state.tys.get(name)?;
                    Some(format!("`{}`", term.ty.to_debug_string(provider)))
                }
                _ => panic!("invalid display type: `{display}`"),
            }
        })();

        replacement.unwrap_or_else(|| {
            valid = false;
            String::new()
        })
    });

    if !valid {
        return None;
    }

    // Collapse line breaks
    let replacement = replacement
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");

    Some(replacement)
}

impl Feedback {
    pub fn render(&self, state: &State<'_, '_>, provider: &FeedbackProvider<'_>) -> Option<String> {
        let mut md = String::new();

        for (index, message) in self.messages.iter().enumerate() {
            let indent = if index == 0 { 0 } else { 1 };

            write!(md, "{}", message.render(state, provider, indent)?).unwrap();

            if index + 1 == self.messages.len() {
                writeln!(md).unwrap();
            }
        }

        Some(md)
    }
}

impl Message {
    pub fn render(
        &self,
        state: &State<'_, '_>,
        provider: &FeedbackProvider<'_>,
        indent: usize,
    ) -> Option<String> {
        const INDENT: &str = "  ";
        let indent_string = |indent| INDENT.repeat(indent);
        let bullet_string = |indent| if indent == 0 { "" } else { "- " };

        let mut md = String::new();

        let render_content = |md: &mut String, content: &Content| {
            match content {
                Content::Documentation(node) => {
                    let _term = state.nodes.get(node)?;
                    todo!("(and only use the first paragraph of the documentation comment)");
                }
                Content::Template(template) => {
                    writeln!(md, "{}", render_template(template, state, provider)?).unwrap();
                }
            }

            Some(())
        };

        match &self {
            Message::Node {
                node: name,
                content,
            } => {
                let term = state.nodes.get(name)?;

                let (node_span, node_source) = provider.node_span_source(term.node);
                write!(
                    md,
                    "{}{}{node_span:?}: `{node_source}`: ",
                    indent_string(indent),
                    bullet_string(indent)
                )
                .unwrap();

                render_content(&mut md, content);
            }
            Message::Trace { trace: name } => {
                let term = state.tys.get(name)?;

                let (node_span, _node_source) = provider.node_span_source(term.node);

                let search = [term.node]
                    .into_iter()
                    .chain(term.related.clone())
                    .collect::<Vec<_>>();

                if let Some(mut common_node) =
                    common_ancestor(state.ctx.relations, search.iter().copied()).next()
                {
                    // If the node directly relates to another node, use that
                    // (eg. show the function instead of its input)
                    let mut skip = BTreeSet::new();
                    let common_node = loop {
                        let neighbors = state
                            .ctx
                            .relations
                            .neighbors_directed(common_node, Direction::Incoming);

                        let Some((next,)) = neighbors.collect_tuple() else {
                            break common_node;
                        };

                        skip.insert(common_node);

                        common_node = next;
                    };

                    let (_, common_source) = provider.node_span_source(common_node);

                    let mut search_sources = search
                        .iter()
                        .filter(|&node| !skip.contains(node))
                        .map(|&node| {
                            let (_span, source) = provider.node_span_source(node);
                            format!("`{source}`")
                        })
                        .collect::<Vec<_>>();

                    let last_search_source = search_sources.pop()?;

                    writeln!(
                        md,
                        "{}{}{:?}: {} and {} need to be the same type because they {} involve `{}`",
                        indent_string(indent),
                        bullet_string(indent),
                        node_span,
                        search_sources.join(", "),
                        last_search_source,
                        if search_sources.len() > 1 {
                            "all"
                        } else {
                            "both"
                        },
                        common_source,
                    )
                    .unwrap();
                }
            }
        }

        Some(md)
    }
}

fn common_ancestor<N: NodeTrait, E: Copy>(
    graph: &DiGraphMap<N, E>,
    nodes: impl IntoIterator<Item = N>,
) -> impl Iterator<Item = N> {
    nodes
        .into_iter()
        .fold(BTreeMap::<N, usize>::new(), |mut result, start| {
            petgraph::visit::depth_first_search(graph, [start], |event| {
                if let petgraph::visit::DfsEvent::Discover(node, _) = event {
                    *result.entry(node).or_default() += 1;
                }
            });

            result
        })
        .into_iter()
        .filter_map(|(node, count)| (count > 1).then_some(node))
}
