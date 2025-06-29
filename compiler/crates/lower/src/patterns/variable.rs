use crate::{Definition, VariableDefinition, Visit, Visitor};
use wipple_compiler_syntax::VariablePattern;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::ConstraintNode,
};

pub const VARIABLE_PATTERN: Rule = Rule::new("variable pattern");

impl Visit for VariablePattern {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            visitor.define_name(
                &self.variable.source,
                Definition::Variable(VariableDefinition { node: id }),
            );

            (
                ConstraintNode {
                    value: id,
                    constraints: vec![Constraint::Ty(Ty::Of(visitor.target()))],
                },
                VARIABLE_PATTERN,
            )
        })
    }
}
