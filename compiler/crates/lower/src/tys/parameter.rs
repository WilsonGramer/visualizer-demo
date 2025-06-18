use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::ParameterType;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::{ConstraintNode, Node, PlaceholderNode},
};

/// A parameter type.
pub const PARAMETER_TYPE: Rule = Rule::new("parameter_type", &[]);

/// An unresolved parameter type.
pub const UNRESOLVED_PARAMETER_TYPE: Rule = Rule::new("unresolved_parameter_type", &[]);

impl Visit for ParameterType {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let existing =
                visitor.resolve_name(&self.name.source, id, PARAMETER_TYPE, |definition| {
                    match definition {
                        Definition::TypeParameter { node } => Some(*node),
                        _ => None,
                    }
                });

            match existing {
                Some(node) => (
                    ConstraintNode {
                        value: visitor.target(),
                        constraints: vec![Constraint::Ty(Ty::Instantiate(node))],
                    }
                    .boxed(),
                    PARAMETER_TYPE,
                ),
                None => {
                    if visitor.implicit_type_parameters {
                        visitor
                            .define_name(&self.name.source, Definition::TypeParameter { node: id });

                        (
                            ConstraintNode {
                                value: visitor.target(),
                                constraints: vec![Constraint::Ty(Ty::Instantiate(id))],
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
