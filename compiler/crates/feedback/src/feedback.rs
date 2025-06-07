use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wipple_compiler_trace::{AnyRule, NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub group: FeedbackGroup,
    pub summary: Message,
    pub details: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackGroup {
    Todo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Message {
    #[serde(flatten)]
    pub kind: MessageKind,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    Node(String),
    Type(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Inherit the node's documentation comment
    Documentation,

    /// Provide a custom message
    Literal(Vec<ContentSegment>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentSegment {
    Text(String),
    Source(String),
    Type(String),
    Rule(String),
}

macro_rules! include_template {
    ($path:literal) => {
        static TEMPLATE: std::sync::LazyLock<$crate::feedback::FeedbackTemplate> =
            std::sync::LazyLock::new(|| {
                $crate::feedback::FeedbackTemplate::new(include_str!($path))
            });
    };
}

pub(crate) use include_template;
use wipple_compiler_typecheck::{constraints::Ty, context::DebugProvider};

#[derive(Debug, Clone)]
pub struct FeedbackTemplate {
    yml: String,
    nodes: HashMap<String, NodeId>,
    tys: HashMap<String, Ty>,
    rules: HashMap<String, AnyRule>,
}

impl FeedbackTemplate {
    pub fn new(yml: &str) -> Self {
        FeedbackTemplate {
            yml: yml.to_string(),
            nodes: Default::default(),
            tys: Default::default(),
            rules: Default::default(),
        }
    }

    pub fn node(mut self, name: &str, node: NodeId) -> Self {
        self.nodes.insert(name.to_string(), node);
        self
    }

    pub fn ty(mut self, name: &str, ty: Ty) -> Self {
        self.tys.insert(name.to_string(), ty);
        self
    }

    pub fn rule(mut self, name: &str, rule: AnyRule) -> Self {
        self.rules.insert(name.to_string(), rule);
        self
    }

    pub fn to_markdown(&self, debug: &DebugProvider<'_>) -> String {
        use std::fmt::Write;

        let item: FeedbackItem = serde_yml::from_str(&self.yml).unwrap();

        let mut md = String::new();

        writeln!(md, "{}", item.summary.to_markdown(self, debug)).unwrap();

        for message in &item.details {
            writeln!(md, "  {}", message.to_markdown(self, debug)).unwrap();
        }

        writeln!(md).unwrap();

        md
    }
}

impl Message {
    pub fn to_markdown(&self, template: &FeedbackTemplate, debug: &DebugProvider<'_>) -> String {
        use std::fmt::Write;

        let mut md = String::new();

        match &self.kind {
            MessageKind::Node(node) => {
                let node = template.nodes.get(node).unwrap();
                let (node_span, node_source) = debug.node_source(*node, Default::default());
                write!(md, "{node_span:?}: `{node_source}`: ").unwrap();
            }
            MessageKind::Type(ty) => {
                let ty = template.tys.get(ty).unwrap();
                write!(md, "for `{}`: ", ty.to_debug_string(debug)).unwrap()
            }
        }

        match &self.content {
            Content::Documentation => todo!(),
            Content::Literal(segments) => {
                for segment in segments {
                    match segment {
                        ContentSegment::Text(s) => {
                            if !md.ends_with(' ')
                                && !s.chars().next().unwrap().is_ascii_punctuation()
                            {
                                write!(md, " ").unwrap();
                            }

                            write!(md, "{s}").unwrap()
                        }
                        ContentSegment::Source(node) => {
                            let node = template.nodes.get(node).unwrap();

                            if !md.ends_with(' ') {
                                write!(md, " ").unwrap();
                            }

                            let (_, source) = debug.node_source(*node, Default::default());
                            write!(md, "`{source}`").unwrap();
                        }
                        ContentSegment::Type(ty) => {
                            let ty = template.tys.get(ty).unwrap();

                            if !md.ends_with(' ') {
                                write!(md, " ").unwrap();
                            }

                            write!(md, "`{}`", ty.to_debug_string(debug)).unwrap()
                        }
                        ContentSegment::Rule(rule) => {
                            let rule = template.rules.get(rule).unwrap();

                            if !md.ends_with(' ') {
                                write!(md, " ").unwrap();
                            }

                            write!(md, "the {rule:?} rule").unwrap()
                        }
                    }
                }
            }
        }

        md
    }
}
