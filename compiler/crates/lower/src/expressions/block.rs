use crate::{Visit, Visitor};
use wipple_compiler_syntax::BlockExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::BlockNode;

pub const BLOCK: Rule = Rule::new("block");

pub const BLOCK_STATEMENT: Rule = Rule::new("block statement");

impl Visit for BlockExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            visitor.push_scope(id);

            let statements = self
                .statements
                .0
                .iter()
                .map(|statement| statement.visit(visitor, (id, BLOCK_STATEMENT)))
                .collect::<Vec<_>>();

            visitor.pop_scope();

            (BlockNode { statements }, BLOCK)
        })
    }
}
