use crate::{Definition, Visit, Visitor};

use wipple_compiler_syntax::TextExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

rule! {
    /// A text literal.
    text;

    /// The `Text` type isn't defined.
    missing_text_type;
}

impl Visit for TextExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, impl Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let text_ty = visitor
                .resolve_name("Text", id, rule::text)
                .and_then(|definition| match definition {
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
                    rule::text.erased(),
                ),
                None => (PlaceholderNode.boxed(), rule::missing_text_type.erased()),
            }
        })
    }
}
