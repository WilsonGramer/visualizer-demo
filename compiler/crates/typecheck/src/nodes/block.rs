use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
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
            ctx.constraints()
                .push(Constraint::Ty(node, Ty::Of(*last_statement)));
        } else {
            ctx.constraints().push(Constraint::Ty(node, Ty::unit()));
        }
    }
}
