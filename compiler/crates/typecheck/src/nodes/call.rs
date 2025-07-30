use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct CallNode {
    pub function: NodeId,
    pub inputs: Vec<NodeId>,
}

impl Node for CallNode {}

impl ToConstraints for CallNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().push(Constraint::Ty(
            self.function,
            Ty::Function {
                inputs: self.inputs.iter().copied().map(Ty::Of).collect(),
                output: Box::new(Ty::Of(node)),
            },
        ));
    }
}
