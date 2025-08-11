use crate::{
    definitions::{Definition, VariableDefinition},
    visitor::{Visit, Visitor},
};
use wipple_db::NodeId;
use wipple_syntax::{Range, VariablePattern};

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
}
