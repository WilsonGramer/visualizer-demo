use crate::{Definition, Visit, Visitor};

use wipple_compiler_syntax::TextExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

/// A text literal.
pub const TEXT: Rule = Rule::new("text");

/// The `Text` type isn't defined.
pub const MISSING_TEXT_TYPE: Rule = Rule::new("missing_text_type");

impl Visit for TextExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let text_ty = visitor.resolve_name("Text", id, TEXT, |definition| match definition {
                Definition::Type { node, .. } => Some(Ty::Named {
                    name: *node,
                    parameters: Vec::new(),
                }),
                _ => None,
            });

            match text_ty {
                Some(text_ty) => (
                    ConstraintNode {
                        value: id,
                        constraints: vec![Constraint::Ty(text_ty)],
                    }
                    .boxed(),
                    TEXT,
                ),
                None => (PlaceholderNode.boxed(), MISSING_TEXT_TYPE),
            }
        })
    }
}
