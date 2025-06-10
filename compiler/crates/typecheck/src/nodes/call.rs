use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, rule};

#[derive(Debug, Clone)]
pub struct CallNode {
    pub function: NodeId,
    pub inputs: Vec<NodeId>,
}

impl Node for CallNode {}

rule! {
    /// A function call.
    call: Extra;
}

impl ToConstraints for CallNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().insert_ty(
            self.function,
            Ty::Function {
                inputs: self.inputs.iter().copied().map(Ty::influences).collect(),
                output: Box::new(Ty::influences(node)),
            },
            rule::call,
        );
    }
}
