use crate::{
    constraints::constraints_for_function,
    visitor::{Visit, Visitor},
};
use wipple_visualizer_syntax::{FunctionExpression, Range};
use wipple_visualizer_typecheck::NodeId;

impl Visit for FunctionExpression {
    fn name(&self) -> &'static str {
        "function"
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
            .map(|input| visitor.child(input, id, "functionInput"))
            .collect::<Vec<_>>();

        let output = visitor.child(self.output.as_ref(), id, "functionOutput");

        visitor.pop_scope();

        visitor.constraints(constraints_for_function(id, inputs, output));
    }

}
