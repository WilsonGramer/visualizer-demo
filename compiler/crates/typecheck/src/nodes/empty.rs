use crate::{
    constraints::{ToConstraints, ToConstraintsContext},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct EmptyNode;

impl Node for EmptyNode {}

impl ToConstraints for EmptyNode {
    fn to_constraints(&self, _node: NodeId, _ctx: &ToConstraintsContext<'_>) {
        // Do nothing
    }
}
