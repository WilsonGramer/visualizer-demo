use crate::{
    constraints::{ToConstraints, ToConstraintsContext},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct PlaceholderNode;

impl Node for PlaceholderNode {}

impl ToConstraints for PlaceholderNode {
    fn to_constraints(&self, _node: NodeId, _ctx: &ToConstraintsContext<'_>) {
        // Do nothing
    }
}
