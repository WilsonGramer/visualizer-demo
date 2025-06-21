use crate::{
    Definition, InstanceDefinition, Visit, Visitor,
    attributes::{AttributeParser, InstanceAttributes},
};
use wipple_compiler_syntax::InstanceDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Bound, Ty},
    nodes::{DefinitionNode, Node, PlaceholderNode},
};

pub const INSTANCE_DEFINITION: Rule = Rule::new("instance definition");

pub const TRAIT_IN_INSTANCE_DEFINITION: Rule = Rule::new("trait in instance definition");

pub const UNRESOLVED_TRAIT_NAME: Rule = Rule::new("unresolved trait name in instance definition");

impl Visit for InstanceDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.range, |visitor, id| {
            let attributes =
                InstanceAttributes::parse(&mut AttributeParser::new(visitor, &self.attributes));

            let Some(((trait_node, trait_parameters), _)) =
                visitor.resolve_name(&self.r#trait.source, id, |definition| match definition {
                    Definition::Trait(definition) => Some((
                        (definition.node, definition.parameters.clone()),
                        TRAIT_IN_INSTANCE_DEFINITION,
                    )),
                    _ => None,
                })
            else {
                return (PlaceholderNode.boxed(), UNRESOLVED_TRAIT_NAME);
            };

            // TODO: Ensure `parameters` and `trait_parameters` have the same
            // length
            let parameters = self
                .parameters
                .iter()
                .map(|ty| {
                    visitor.with_implicit_type_parameters(|visitor| {
                        visitor.with_target(id, |visitor| ty.visit(visitor, None))
                    })
                })
                .map(Ty::Of)
                .collect::<Vec<_>>();

            // TODO
            let constraints = self.constraints.iter().map(|_| todo!()).collect::<Vec<_>>();

            visitor.define_instance(InstanceDefinition {
                node: id,
                comments: Vec::new(),
                attributes,
                bound: Bound {
                    source: id,
                    tr: trait_node,
                    parameters,
                },
                constraints,
            });

            let value = self
                .value
                .as_ref()
                .map(|value| visitor.with_target(id, |visitor| value.visit(visitor, None)))
                .map_or_else(
                    || PlaceholderNode.boxed(),
                    |node| {
                        DefinitionNode {
                            definition: node,
                            // Constraints will be checked on the instance
                            // itself
                            constraints: Vec::new(),
                        }
                        .boxed()
                    },
                );

            (value, INSTANCE_DEFINITION)
        })
    }
}
