use crate::{Definition, Visit, Visitor};
use std::collections::BTreeMap;
use wipple_compiler_syntax::BoundConstraint;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::Substitutions,
    nodes::{Annotation, EmptyNode},
};

pub static RESOLVED_TRAIT_IN_BOUND: Rule = Rule::new("trait in bound");
pub static UNRESOLVED_TRAIT_IN_BOUND: Rule = Rule::new("unresolved trait in bound");
pub static PARAMETER_IN_BOUND: Rule = Rule::new("parameter in bound");

impl Visit for BoundConstraint {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let Some(((trait_node, trait_parameters), rule)) =
                visitor.resolve_name(&self.r#trait.value, id, |definition| match definition {
                    Definition::Trait(definition) => Some((
                        (definition.node, definition.parameters.clone()),
                        RESOLVED_TRAIT_IN_BOUND,
                    )),
                    _ => None,
                })
            else {
                return (EmptyNode, UNRESOLVED_TRAIT_IN_BOUND);
            };

            let parameters = self
                .parameters
                .iter()
                .map(|ty| ty.visit(visitor, (id, PARAMETER_IN_BOUND)));

            // TODO: Ensure `parameters` has the right length
            let substitutions = trait_parameters
                .into_iter()
                .zip(parameters)
                .collect::<BTreeMap<_, _>>();

            visitor
                .current_definition()
                .annotations
                .push(Annotation::Bound {
                    node: Some(id),
                    tr: trait_node,
                    substitutions: Substitutions::from(substitutions),
                });

            (EmptyNode, rule)
        })
    }
}
