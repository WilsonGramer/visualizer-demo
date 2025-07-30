use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct TupleNode {
    pub elements: Vec<NodeId>,
}

impl Node for TupleNode {}

impl ToConstraints for TupleNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().push(Constraint::Ty(
            node,
            Ty::Tuple {
                elements: self.elements.iter().copied().map(Ty::Of).collect(),
            },
        ));
    }
}
