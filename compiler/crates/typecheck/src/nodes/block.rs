use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, rule};

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub statements: Vec<NodeId>,
}

impl Node for BlockNode {}

rule! {
    /// A statement whose value is discarded because it's not the last in the
    /// block.
    block_statement;

    /// The last statement in a block.
    block_last_statement;

    /// An empty block.
    empty_block;
}

impl ToConstraints for BlockNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        if let Some((last_statement, statements)) = self.statements.split_last() {
            for statement in statements {
                ctx.constraints()
                    .insert_extra(*statement, rule::block_statement);
            }

            ctx.constraints().insert_ty(
                node,
                Ty::influenced_by(*last_statement),
                rule::block_last_statement,
            );
        } else {
            ctx.constraints()
                .insert_ty(node, Ty::unit(), rule::empty_block);
        }
    }
}
