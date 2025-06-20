use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::VariableNameExpression;
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};
use wipple_compiler_typecheck::nodes::{DefinitionNode, Node, PlaceholderNode};

/// A variable name expression.
pub const VARIABLE_NAME: Rule = Rule::new("variable_name", &[RuleCategory::Expression]);

/// A variable name that resolved to a value.
pub const RESOLVED_VARIABLE_NAME: Rule =
    Rule::new("resolved_variable_name", &[RuleCategory::Expression]);

/// A variable name that resolved to a value.
pub const RESOLVED_CONSTANT_NAME: Rule =
    Rule::new("resolved_constant_name", &[RuleCategory::Expression]);

/// A variable name that was not resolved to a value.
pub const UNRESOLVED_VARIABLE_NAME: Rule =
    Rule::new("unresolved_variable_name", &[RuleCategory::Expression]);

impl Visit for VariableNameExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            if let Some(((definition, constraints), rule)) =
                visitor.resolve_name(&self.variable.source, id, |definition| match definition {
                    Definition::Variable { node, .. } => {
                        Some(((*node, Vec::new()), RESOLVED_VARIABLE_NAME))
                    }
                    Definition::Constant {
                        node, constraints, ..
                    } => Some(((*node, constraints.clone()), RESOLVED_CONSTANT_NAME)),
                    _ => None,
                })
            {
                (
                    DefinitionNode {
                        definition,
                        constraints,
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
