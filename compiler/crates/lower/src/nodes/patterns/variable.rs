use crate::{
    definitions::{Definition, VariableDefinition},
    visitor::{Visit, Visitor},
};
use wipple_compiler_syntax::{Range, VariablePattern};
use wipple_compiler_trace::NodeId;

impl Visit for VariablePattern {
    fn name(&self) -> &'static str {
        "variablePattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        visitor.define_name(
            &self.variable.value,
            Definition::Variable(VariableDefinition { node: id }),
        );
    }

    fn is_typed(&self) -> bool {
        true
    }
}
