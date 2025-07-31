use crate::{
    constraints::constraints_for_call,
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{FunctionType, Range};
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for FunctionType {
    fn rule(&self) -> Rule {
        "function type".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let inputs = self
            .inputs
            .0
            .iter()
            .map(|input| visitor.child(input, id, "function type input"))
            .collect::<Vec<_>>();

        let output = visitor.child(self.output.as_ref(), id, "function type output");

        visitor.constraints(constraints_for_call(id, inputs, output));
    }
}
