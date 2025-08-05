use crate::{Fact, NodeId, Span};

pub trait DisplayProvider {
    fn node_facts(&self, node: NodeId) -> &[Fact];
    fn node_span_source(&self, node: NodeId) -> (Span, String);
    fn node_comments(&self, node: NodeId) -> Option<String>;
}
