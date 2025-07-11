use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NamedType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const RESOLVED_NAMED_TYPE: Rule = Rule::new("resolved named type");

pub const UNRESOLVED_NAMED_TYPE: Rule = Rule::new("unresolved named type");

pub const NAME_IN_NAMED_TYPE: Rule = Rule::new("name in named type");

pub const PARAMETER_IN_NAMED_TYPE: Rule = Rule::new("parameter in named type");

impl Visit for NamedType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let Some(((type_node, type_parameters), rule)) =
                visitor.resolve_name(&self.name.value, id, |definition| match definition {
                    Definition::Type(definition) => Some((
                        (definition.node, definition.parameters.clone()),
                        NAME_IN_NAMED_TYPE,
                    )),
                    _ => None,
                })
            else {
                return (PlaceholderNode.boxed(), UNRESOLVED_NAMED_TYPE);
            };

            // TODO: Ensure `type_parameters` has length of 0

            (
                ConstraintNode {
                    value: visitor.target(),
                    constraints: vec![Constraint::Ty(Ty::Named {
                        name: type_node,
                        parameters: Vec::new(), // TODO: Fill with placeholders if necessary
                    })],
                }
                .boxed(),
                rule,
            )
        })
    }
}
