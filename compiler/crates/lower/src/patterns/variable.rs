use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::VariablePattern;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::DefinitionNode;

rule! {
    /// A tuple pattern.
    variable_pattern: Typed;

    /// The target of a tuple pattern.
    variable_pattern_target: Typed;
}

impl Visit for VariablePattern {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
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
                rule::variable_pattern,
            )
        })
    }
}
