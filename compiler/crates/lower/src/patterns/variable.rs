use crate::{Definition, VariableDefinition, Visit, Visitor};
use wipple_compiler_syntax::VariablePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::EmptyNode;

pub static VARIABLE_PATTERN: Rule = Rule::new("variable pattern");

impl Visit for VariablePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            visitor.define_name(
                &self.variable.value,
                Definition::Variable(VariableDefinition { node: id }),
            );

            (EmptyNode, VARIABLE_PATTERN)
        })
    }
}
