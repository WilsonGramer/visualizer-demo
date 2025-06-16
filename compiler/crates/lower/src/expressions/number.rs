use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NumberExpression;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

rule! {
    /// A number literal.
    number: Typed;

    /// The `Number` type isn't defined.
    missing_number_type: Typed;
}

impl Visit for NumberExpression {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let number_ty =
                visitor.resolve_name("Number", id, rule::number, |definition| match definition {
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
                    rule::number.erased(),
                ),
                None => (PlaceholderNode.boxed(), rule::missing_number_type.erased()),
            }
        })
    }
}
