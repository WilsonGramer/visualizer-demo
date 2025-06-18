use crate::{Definition, Visit, Visitor, tys::parameter::PARAMETER_TYPE};
use wipple_compiler_syntax::NamedType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

/// A resolved named type.
pub const RESOLVED_NAMED_TYPE: Rule = Rule::new("resolved_named_type", &[]);

/// An unresolved named type.
pub const UNRESOLVED_NAMED_TYPE: Rule = Rule::new("unresolved_named_type", &[]);

/// The name in a named type.
pub const NAME_IN_NAMED_TYPE: Rule = Rule::new("name_in_named_type", &[]);

/// A parameter in a named type.
pub const PARAMETER_IN_NAMED_TYPE: Rule = Rule::new("parameter_in_named_type", &[]);

impl Visit for NamedType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let Some((type_node, type_parameters)) =
                visitor.resolve_name(&self.name.source, id, NAME_IN_NAMED_TYPE, |definition| {
                    match definition {
                        Definition::Type {
                            node, parameters, ..
                        } => Some((*node, parameters.clone())),
                        _ => None,
                    }
                })
            else {
                return (PlaceholderNode.boxed(), UNRESOLVED_NAMED_TYPE);
            };

            // TODO: Ensure `parameters` and `type_parameters` have the same
            // length
            let parameters = self
                .parameters
                .iter()
                .map(|ty| {
                    let input_target =
                        visitor.root_placeholder_node(ty.range(), PARAMETER_IN_NAMED_TYPE);

                    visitor.with_target(input_target, |visitor| {
                        Ty::Of(ty.visit(visitor, Some((id, PARAMETER_IN_NAMED_TYPE))))
                    })
                })
                .collect();

            (
                ConstraintNode {
                    value: visitor.target(),
                    constraints: vec![Constraint::Ty(Ty::Named {
                        name: type_node,
                        parameters,
                    })],
                }
                .boxed(),
                RESOLVED_NAMED_TYPE,
            )
        })
    }
}
