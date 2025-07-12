use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, Rule};

pub static UNIT: Rule = Rule::new("unit");

#[derive(Debug, Clone)]
pub struct UnitNode {}

impl Node for UnitNode {}

impl ToConstraints for UnitNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints().insert_ty(node, Ty::unit(), UNIT);
    }
}
