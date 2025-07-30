use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct UnitNode {}

impl Node for UnitNode {}

impl ToConstraints for UnitNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().push(Constraint::Ty(node, Ty::unit()));
    }
}
