use itertools::Itertools;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::{fmt::Write, sync::LazyLock};
use wipple_compiler_trace::{AnyRule, NodeId, Rule};
use wipple_compiler_typecheck::{constraints::Ty, context::FeedbackProvider};

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
    #[serde(default)]
    pub trace: TraceStyle,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraceStyle {
    Related,
    #[default]
    Because,
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

#[derive(Debug, Default)]
pub struct TermCounts {
    pub nodes: usize,
    pub tys: usize,
}

#[derive(Debug)]
pub struct TermsIter {
    relations: std::vec::IntoIter<NodeTerm>,
    visited_relations: HashMap<String, NodeTerm>,
    tys: std::vec::IntoIter<TyTerm>,
    visited_tys: HashMap<String, TyTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeTerm {
    pub node: NodeId,
    pub rule: AnyRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TyTerm {
    pub ty: Ty,
    pub related: Vec<NodeTerm>,
}

impl TermsIter {
    pub fn new(relations: Vec<NodeTerm>, tys: Vec<TyTerm>) -> Self {
        TermsIter {
            relations: relations.into_iter(),
            visited_relations: Default::default(),
            tys: tys.into_iter(),
            visited_tys: Default::default(),
        }
    }

    pub fn relation(&mut self, name: &str) -> NodeTerm {
        self.visited_relations
            .entry(name.to_string())
            .or_insert_with(|| self.relations.next().unwrap())
            .clone()
    }

    pub fn ty(&mut self, name: &str) -> TyTerm {
        self.visited_tys
            .entry(name.to_string())
            .or_insert_with(|| self.tys.next().unwrap())
            .clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub node: NodeTerm,
    pub nodes: BTreeMap<String, NodeTerm>,
    pub tys: BTreeMap<String, TyTerm>,
}

impl State {
    pub fn new(node: NodeTerm) -> Self {
        State {
            node,
            nodes: Default::default(),
            tys: Default::default(),
        }
    }
}

fn render_template(
    template: &str,
    state: &State,
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
    pub fn render(&self, state: &State, provider: &FeedbackProvider<'_>) -> Option<String> {
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
        state: &State,
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

                let mut related = term.related.clone();
                for term in &term.related {
                    // TODO: Currently disabled
                    const LEVEL: usize = 0;

                    collect_related(provider, term.node, &mut related, LEVEL);
                }

                for related in related {
                    let (_, related_source) = provider.node_span_source(related.node);

                    write!(
                        md,
                        "{}{}",
                        indent_string(indent + 1),
                        bullet_string(indent + 1),
                    )
                    .unwrap();

                    match self.options.trace {
                        TraceStyle::Related => writeln!(
                            md,
                            "See the related {} in `{}`.",
                            related.rule.name(),
                            related_source,
                        )
                        .unwrap(),
                        TraceStyle::Because => writeln!(
                            md,
                            "...because of {} in `{}`.",
                            related.rule.name(),
                            related_source,
                        )
                        .unwrap(),
                    }
                }
            }
        }

        Some(md)
    }
}

fn collect_related(
    provider: &FeedbackProvider<'_>,
    node: NodeId,
    influences: &mut Vec<NodeTerm>,
    level: usize,
) {
    if level == 0 {
        return;
    }

    for (node, rule) in provider.related_nodes(node) {
        let term = NodeTerm { node, rule };

        if !influences.contains(&term) {
            influences.push(term.clone());
            collect_related(provider, term.node, influences, level - 1);
        }
    }
}
