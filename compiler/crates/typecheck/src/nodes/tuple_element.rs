use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

#[derive(Debug, Clone)]
pub struct TupleElementNode {
    pub index: usize,
    pub count: usize,
    pub target: NodeId,
}

impl Node for TupleElementNode {}

impl ToConstraints for TupleElementNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        let mut elements = vec![Ty::Any; self.count];
        elements[self.index] = Ty::Of(node);

        ctx.constraints()
            .insert_ty(self.target, Ty::Tuple { elements });
    }
}
