use crate::{
    constraints::constraints_for_block,
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{BlockExpression, Range, Statement};
use wipple_visualizer_typecheck::NodeId;

impl Visit for BlockExpression {
    fn name(&self) -> &'static str {
        "block"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.push_scope(id);

        let statements = self
            .statements
            .0
            .iter()
            .filter(|statement| !matches!(statement, Statement::Empty(_)))
            .map(|statement| visitor.child(statement, id, "blockStatement"))
            .collect::<Vec<_>>();

        visitor.pop_scope();

        visitor.constraints(constraints_for_block(statements, id));
    }

}
