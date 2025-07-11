use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::VariableExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const VARIABLE_NAME: Rule = Rule::new("variable name");

pub const RESOLVED_VARIABLE_NAME: Rule = Rule::new("resolved variable name");

pub const RESOLVED_CONSTANT_NAME: Rule = Rule::new("resolved constant name");

pub const UNRESOLVED_VARIABLE_NAME: Rule = Rule::new("unresolved variable name");

impl Visit for VariableExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            if let Some((constraint, rule)) =
                visitor.resolve_name(&self.variable.value, id, |definition| match definition {
                    Definition::Variable(definition) => Some((
                        Constraint::Ty(Ty::Of(definition.node)),
                        RESOLVED_VARIABLE_NAME,
                    )),
                    Definition::Constant(definition) => {
                        Some((Constraint::Generic(definition.node), RESOLVED_CONSTANT_NAME))
                    }
                    _ => None,
                })
            {
                (
                    ConstraintNode {
                        value: id,
                        constraints: vec![constraint],
                    }
                    .boxed(),
                    rule,
                )
            } else {
                (PlaceholderNode.boxed(), UNRESOLVED_VARIABLE_NAME)
            }
        })
    }
}
