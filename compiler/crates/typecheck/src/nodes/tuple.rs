use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, Rule};

pub static TUPLE: Rule = Rule::new("tuple");

#[derive(Debug, Clone)]
pub struct TupleNode {
    pub elements: Vec<NodeId>,
}

impl Node for TupleNode {}

impl ToConstraints for TupleNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().insert_ty(
            node,
            Ty::Tuple {
                elements: self.elements.iter().copied().map(Ty::of).collect(),
            },
            TUPLE,
        );
    }
}
