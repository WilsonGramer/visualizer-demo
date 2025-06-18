use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::VariablePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::DefinitionNode;

/// A tuple pattern.
pub const VARIABLE_PATTERN: Rule = Rule::new("variable_pattern", &[]);

/// The target of a tuple pattern.
pub const VARIABLE_PATTERN_TARGET: Rule = Rule::new("variable_pattern_target", &[]);

impl Visit for VariablePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, _id| {
            visitor.define_name(
                &self.variable.source,
                Definition::Variable {
                    node: visitor.parent(),
                },
            );

            (
                DefinitionNode {
                    definition: visitor.parent(),
                    constraints: Vec::new(),
                },
                VARIABLE_PATTERN,
            )
        })
    }
}
