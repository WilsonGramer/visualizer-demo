use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::NamedType;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

rule! {
    /// A resolved named type.
    resolved_named_type: Typed;

    /// An unresolved named type.
    unresolved_named_type: Typed;

    /// The name in a named type.
    name_in_named_type: Typed;

    /// A parameter in a named type.
    parameter_in_named_type: Typed;
}

impl Visit for NamedType {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let Some((type_node, type_parameters)) = visitor
                .resolve_name(&self.name.source, id, rule::name_in_named_type)
                .and_then(|definition| match definition {
                    Definition::Type {
                        node, parameters, ..
                    } => Some((*node, parameters.clone())),
                    _ => None,
                })
            else {
                return (
                    PlaceholderNode.boxed(),
                    rule::unresolved_named_type.erased(),
                );
            };

            // TODO: Ensure `parameters` and `type_parameters` have the same
            // length
            let parameters = self
                .parameters
                .iter()
                .map(|ty| Ty::Of(ty.visit(visitor, Some((id, rule::parameter_in_named_type)))))
                .collect();

            (
                ConstraintNode {
                    value: id,
                    constraints: vec![Constraint::Ty(Ty::Named {
                        name: type_node,
                        parameters,
                    })],
                }
                .boxed(),
                rule::resolved_named_type.erased(),
            )
        })
    }
}
