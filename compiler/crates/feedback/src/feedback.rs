use crate::selectors::State;
use itertools::Itertools;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, sync::LazyLock};
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
pub struct Message {
    #[serde(flatten)]
    pub kind: MessageKind,
    #[serde(flatten)]
    pub options: MessageOptions,
    #[serde(flatten)]
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct MessageOptions {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    Node(String),
    Type(String),
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

        let render_content = |md: &mut String| {
            match &self.content {
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

        match &self.kind {
            MessageKind::Node(name) => {
                let term = state.nodes.get(name)?;

                let (node_span, node_source) = provider.node_span_source(term.node);
                write!(
                    md,
                    "{}{}{node_span:?}: `{node_source}`: ",
                    indent_string(indent),
                    bullet_string(indent)
                )
                .unwrap();
                render_content(&mut md);
            }
            MessageKind::Type(name) => {
                write!(md, "{}{}", indent_string(indent), bullet_string(indent)).unwrap();

                render_content(&mut md);

                let term = state.tys.get(name)?;

                for &related_node in &term.related {
                    let (related_node_span, related_node_source) =
                        provider.node_span_source(related_node);

                    write!(
                        md,
                        "{}{}{:?}: ...because of `{}`",
                        indent_string(indent + 1),
                        bullet_string(indent + 1),
                        related_node_span,
                        related_node_source,
                    )
                    .unwrap();

                    // TODO: Find the common ancestor of all related nodes (related by syntax)

                    writeln!(md).unwrap();
                }
            }
        }

        Some(md)
    }
}
