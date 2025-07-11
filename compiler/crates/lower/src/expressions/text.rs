use crate::{Definition, Visit, Visitor};

use wipple_compiler_syntax::TextExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const TEXT: Rule = Rule::new("text");

pub const MISSING_TEXT_TYPE: Rule = Rule::new("missing text type");

impl Visit for TextExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let text_ty = visitor.resolve_name("Text", id, |definition| match definition {
                Definition::Type(definition) => Some((
                    Ty::Named {
                        name: definition.node,
                        parameters: Vec::new(),
                    },
                    TEXT,
                )),
                _ => None,
            });

            match text_ty {
                Some((text_ty, rule)) => (
                    ConstraintNode {
                        value: id,
                        constraints: vec![Constraint::Ty(text_ty)],
                    }
                    .boxed(),
                    rule,
                ),
                None => (PlaceholderNode.boxed(), MISSING_TEXT_TYPE),
            }
        })
    }
}
