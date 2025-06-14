use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct ConstraintNode {
    pub value: NodeId,
    pub constraints: Vec<Constraint>,
}

impl Node for ConstraintNode {}

impl ToConstraints for ConstraintNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints()
            .extend(self.value, self.constraints.clone());

        ctx.constraints()
            .extend(node, vec![Constraint::Ty(Ty::Of(self.value))]);
    }
}
