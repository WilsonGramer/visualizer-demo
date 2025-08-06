use crate::{
    constraints::constraints_for_call,
    visitor::{Visit, Visitor},
};
use wipple_db::NodeId;
use wipple_syntax::{FunctionType, Range};

impl Visit for FunctionType {
    fn name(&self) -> &'static str {
        "functionType"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let inputs = self
            .inputs
            .0
            .iter()
            .map(|input| visitor.child(input, id, "functionTypeInput"))
            .collect::<Vec<_>>();

        let output = visitor.child(self.output.as_ref(), id, "functionTypeOutput");

        visitor.constraints(constraints_for_call(id, inputs, output));
    }
}
