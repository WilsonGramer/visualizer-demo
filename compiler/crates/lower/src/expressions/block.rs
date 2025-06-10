use crate::{Visit, Visitor};
use wipple_compiler_syntax::BlockExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::BlockNode;

rule! {
    /// A block expression.
    block: Typed;

    /// A statement in a block.
    block_statement: Typed;
}

impl Visit for BlockExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            visitor.push_scope();

            let statements = self
                .statements
                .iter()
                .map(|statement| statement.visit(visitor, Some((id, rule::block_statement))))
                .collect::<Vec<_>>();

            visitor.pop_scope();

            (BlockNode { statements }, rule::block)
        })
    }
}
