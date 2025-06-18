use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NumberExpression;
use wipple_compiler_trace::{NodeId, Rule, RuleCategory};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

/// A number literal.
pub const NUMBER: Rule = Rule::new("number", &[]);

/// The `Number` type isn't defined.
pub const MISSING_NUMBER_TYPE: Rule = Rule::new("missing_number_type", &[]);

impl Visit for NumberExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            let number_ty =
                visitor.resolve_name("Number", id, NUMBER, |definition| match definition {
                    Definition::Type { node, .. } => Some(Ty::Named {
                        name: *node,
                        parameters: Vec::new(),
                    }),
                    _ => None,
                });

            match number_ty {
                Some(number_ty) => (
                    ConstraintNode {
                        value: id,
                        constraints: vec![Constraint::Ty(number_ty)],
                    }
                    .boxed(),
                    NUMBER,
                ),
                None => (PlaceholderNode.boxed(), MISSING_NUMBER_TYPE),
            }
        })
    }
}
