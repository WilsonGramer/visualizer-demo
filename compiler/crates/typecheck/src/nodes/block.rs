use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, Rule};

pub static BLOCK_LAST_STATEMENT: Rule = Rule::new("last statement in block");
pub static EMPTY_BLOCK: Rule = Rule::new("empty block");

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub statements: Vec<NodeId>,
}

impl Node for BlockNode {}

impl ToConstraints for BlockNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        if let Some(last_statement) = self.statements.last() {
            ctx.constraints()
                .insert_ty(node, Ty::of(*last_statement), BLOCK_LAST_STATEMENT);
        } else {
            ctx.constraints().insert_ty(node, Ty::unit(), EMPTY_BLOCK);
        }
    }
}
