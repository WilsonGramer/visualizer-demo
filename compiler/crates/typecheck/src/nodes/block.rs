use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub statements: Vec<NodeId>,
}

impl Node for BlockNode {}

impl ToConstraints for BlockNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        if let Some(last_statement) = self.statements.last() {
            ctx.constraints().insert_ty(node, Ty::Of(*last_statement));
        } else {
            ctx.constraints().insert_ty(node, Ty::unit());
        }
    }
}
