use crate::{Definition, TypeParameterDefinition, Visit, Visitor};
use wipple_compiler_syntax::ParameterType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

pub const PARAMETER_TYPE: Rule = Rule::new("parameter type");

pub const UNRESOLVED_PARAMETER_TYPE: Rule = Rule::new("unresolved parameter type");

impl Visit for ParameterType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let existing =
                visitor.resolve_name(&self.name.source, id, |definition| match definition {
                    Definition::TypeParameter(definition) => {
                        Some((definition.node, PARAMETER_TYPE))
                    }
                    _ => None,
                });

            match existing {
                Some((node, rule)) => (
                    ConstraintNode {
                        value: visitor.target(),
                        constraints: vec![Constraint::Ty(Ty::Of(node))],
                    }
                    .boxed(),
                    rule,
                ),
                None => {
                    if visitor.implicit_type_parameters {
                        visitor.define_name(
                            &self.name.source,
                            Definition::TypeParameter(TypeParameterDefinition { node: id }),
                        );

                        (
                            ConstraintNode {
                                value: visitor.target(),
                                constraints: vec![Constraint::Ty(Ty::Of(id))],
                            }
                            .boxed(),
                            PARAMETER_TYPE,
                        )
                    } else {
                        (PlaceholderNode.boxed(), UNRESOLVED_PARAMETER_TYPE)
                    }
                }
            }
        })
    }
}
