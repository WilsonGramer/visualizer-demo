use crate::{Visit, Visitor};
use wipple_compiler_syntax::BlockExpression;
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};
use wipple_compiler_typecheck::nodes::BlockNode;

/// A block expression.
pub const BLOCK: Rule = Rule::new("block", &[RuleCategory::Expression]);

/// A statement in a block.
pub const BLOCK_STATEMENT: Rule = Rule::new("block_statement", &[RuleCategory::Expression]);

impl Visit for BlockExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            visitor.push_scope(id);

            let statements = self
                .statements
                .iter()
                .map(|statement| statement.visit(visitor, Some((id, BLOCK_STATEMENT))))
                .collect::<Vec<_>>();

            visitor.pop_scope();

            (BlockNode { statements }, BLOCK)
        })
    }
}
