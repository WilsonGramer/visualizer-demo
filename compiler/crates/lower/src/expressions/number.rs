use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NumberExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const NUMBER: Rule = Rule::new("number");

pub const MISSING_NUMBER_TYPE: Rule = Rule::new("missing number type");

impl Visit for NumberExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, &self.range, |visitor, id| {
            let number_ty = visitor.resolve_name("Number", id, |definition| match definition {
                Definition::Type(definition) => Some((
                    Ty::Named {
                        name: definition.node,
                        parameters: Vec::new(),
                    },
                    NUMBER,
                )),
                _ => None,
            });

            match number_ty {
                Some((number_ty, rule)) => (
                    ConstraintNode {
                        value: id,
                        constraints: vec![Constraint::Ty(number_ty)],
                    }
                    .boxed(),
                    rule,
                ),
                None => (PlaceholderNode.boxed(), MISSING_NUMBER_TYPE),
            }
        })
    }
}
