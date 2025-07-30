use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub inputs: Vec<NodeId>,
    pub output: NodeId,
}

impl Node for FunctionNode {}

impl ToConstraints for FunctionNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().push(Constraint::Ty(
            node,
            Ty::Function {
                inputs: self.inputs.iter().copied().map(Ty::Of).collect(),
                output: Box::new(Ty::Of(self.output)),
            },
        ));
    }
}
