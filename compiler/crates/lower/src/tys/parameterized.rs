use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::ParameterizedType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const RESOLVED_PARAMETERIZED_TYPE: Rule = Rule::new("resolved parameterized type");

pub const UNRESOLVED_PARAMETERIZED_TYPE: Rule = Rule::new("unresolved parameterized type");

pub const NAME_IN_PARAMETERIZED_TYPE: Rule = Rule::new("name in parameterized type");

pub const PARAMETER_IN_PARAMETERIZED_TYPE: Rule = Rule::new("parameter in parameterized type");

impl Visit for ParameterizedType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let Some(((type_node, type_parameters), rule)) =
                visitor.resolve_name(&self.name.value, id, |definition| match definition {
                    Definition::Type(definition) => Some((
                        (definition.node, definition.parameters.clone()),
                        NAME_IN_PARAMETERIZED_TYPE,
                    )),
                    _ => None,
                })
            else {
                return (PlaceholderNode.boxed(), UNRESOLVED_PARAMETERIZED_TYPE);
            };

            // TODO: Ensure `parameters` and `type_parameters` have the same
            // length
            let parameters = self
                .parameters
                .iter()
                .map(|ty| {
                    let node = visitor.node(
                        (id, PARAMETER_IN_PARAMETERIZED_TYPE),
                        ty.0.range(),
                        |visitor, target| {
                            visitor.with_target(target, |visitor| {
                                Ty::Of(ty.0.visit(visitor, (id, PARAMETER_IN_PARAMETERIZED_TYPE)))
                            });

                            (PlaceholderNode, PARAMETER_IN_PARAMETERIZED_TYPE)
                        },
                    );

                    Ty::Of(node)
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
                rule,
            )
        })
    }
}
