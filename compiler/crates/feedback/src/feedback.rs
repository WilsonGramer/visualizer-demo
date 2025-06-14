use crate::selectors::State;
use itertools::Itertools;
use petgraph::{prelude::UnGraphMap, visit::Bfs};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, sync::LazyLock};
use wipple_compiler_trace::{AnyRule, NodeId, Rule};
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

                let (node_span, node_source) = provider.node_span_source(term.node);

                writeln!(
                    md,
                    "{}{}{:?}: `{}` must be `{}` because of:",
                    indent_string(indent),
                    bullet_string(indent),
                    node_span,
                    node_source,
                    term.ty.to_debug_string(provider),
                )
                .unwrap();

                let relations_undirected =
                    UnGraphMap::<NodeId, AnyRule>::from_edges(state.ctx.relations.all_edges());

                let trace = term
                    .related
                    .iter()
                    .filter_map(|&related_node| {
                        // TODO: Move to separate function

                        let mut visited_from_term = Vec::new();
                        let mut visited_from_related = Vec::new();
                        let mut from_term = Bfs::new(&relations_undirected, term.node);
                        let mut from_related = Bfs::new(&relations_undirected, related_node);
                        let (common_node, path) = loop {
                            let Some(term_current) = from_term.next(&relations_undirected) else {
                                break None;
                            };

                            let Some(related_current) = from_related.next(&relations_undirected)
                            else {
                                break None;
                            };

                            if term_current == related_current
                                || visited_from_related.iter().contains(&term_current)
                            {
                                visited_from_related.push(related_current);
                                break Some((term_current, visited_from_related));
                            } else if visited_from_term.iter().contains(&related_current) {
                                visited_from_term.push(term_current);
                                break Some((related_current, visited_from_term));
                            }

                            visited_from_term.push(term_current);
                            visited_from_related.push(related_current);
                        }?;

                        let path = path
                            .into_iter()
                            .chain([common_node])
                            .tuple_windows()
                            .filter_map(|(from, to)| {
                                Some((from, to, relations_undirected.edge_weight(from, to)?))
                            })
                            .filter(|(_, _, rule)| !rule.kind().is_hidden())
                            .collect::<Vec<_>>();

                        Some((related_node, path))
                    })
                    .min_by_key(|(_, path)| path.len());

                if let Some((related_node, path)) = trace {
                    let (related_node_span, related_node_source) =
                        provider.node_span_source(related_node);

                    write!(
                        md,
                        "{}{}{:?}: `{}`",
                        indent_string(indent + 1),
                        bullet_string(indent + 1),
                        related_node_span,
                        related_node_source
                    )
                    .unwrap();

                    for (from, to, rule) in path {
                        let (_from_span, from_source) = provider.node_span_source(from);
                        let (_to_span, to_source) = provider.node_span_source(to);

                        write!(
                            md,
                            ", via `{}` as {} to `{}`",
                            from_source,
                            rule.name(),
                            to_source
                        )
                        .unwrap();
                    }

                    writeln!(md).unwrap();
                }
            }
        }

        Some(md)
    }
}
