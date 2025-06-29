use crate::{
    Definition, TraitDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TraitAttributes},
};
use wipple_compiler_syntax::TraitDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::PlaceholderNode,
};

pub const TRAIT_DEFINITION: Rule = Rule::new("trait definition");

pub const PARAMETER_IN_TRAIT_DEFINITION: Rule = Rule::new("parameter in trait definition");

pub const TYPE_IN_TRAIT_DEFINITION: Rule = Rule::new("type in trait definition");

impl Visit for TraitDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, &self.name.range, |visitor, id| {
            let attributes =
                TraitAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let parameters = self
                .parameters
                .iter()
                .map(|parameter| {
                    let node = visitor
                        .placeholder_node((id, PARAMETER_IN_TRAIT_DEFINITION), &parameter.range);

                    visitor.define_name(
                        &parameter.source,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    node
                })
                .collect::<Vec<_>>();

            let ty = self.r#type.as_ref().map(|ty| {
                visitor.node(
                    (id, TYPE_IN_TRAIT_DEFINITION),
                    ty.range(),
                    |visitor, node| {
                        visitor.with_target(node, |visitor| {
                            ty.visit(visitor, (id, TYPE_IN_TRAIT_DEFINITION))
                        });

                        (PlaceholderNode, TYPE_IN_TRAIT_DEFINITION)
                    },
                )
            });

            visitor.define_name(
                &self.name.source,
                Definition::Trait(TraitDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters: parameters.clone(),
                    constraints: Vec::from_iter(ty.map(|node| Constraint::Ty(Ty::Of(node)))),
                }),
            );

            (PlaceholderNode, TRAIT_DEFINITION)
        })
    }
}
