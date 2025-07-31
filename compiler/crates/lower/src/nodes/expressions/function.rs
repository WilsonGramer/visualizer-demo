use crate::{
    constraints::constraints_for_function,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{FunctionExpression, Range};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for FunctionExpression {
    fn rule(&self) -> Rule {
        "function".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.push_scope(id);

        let inputs = self
            .inputs
            .0
            .iter()
            .map(|input| visitor.child(input, id, "function input"))
            .collect::<Vec<_>>();

        let output = visitor.child(self.output.as_ref(), id, "function output");

        visitor.pop_scope();

        visitor.constraints(constraints_for_function(id, inputs, output));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
