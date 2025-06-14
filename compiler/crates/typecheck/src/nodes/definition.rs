use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct DefinitionNode {
    pub definition: NodeId,
    pub constraints: Vec<Constraint>,
}

impl Node for DefinitionNode {}

impl ToConstraints for DefinitionNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().insert_ty(node, Ty::Of(self.definition));

        ctx.constraints().extend(node, self.constraints.clone());
    }
}
