use crate::{
    constraints::constraints_for_block,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{BlockExpression, Range, Statement};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for BlockExpression {
    fn rule(&self) -> Rule {
        "block".into()
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
            .map(|statement| visitor.child(statement, id, "block statement"))
            .collect::<Vec<_>>();

        visitor.pop_scope();

        visitor.constraints(constraints_for_block(statements, id));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
