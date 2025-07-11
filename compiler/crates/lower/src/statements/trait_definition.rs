use crate::{
    Definition, TraitDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TraitAttributes},
};
use wipple_compiler_syntax::TraitDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Bound, Constraint, Ty},
    nodes::PlaceholderNode,
};

pub const TRAIT_DEFINITION: Rule = Rule::new("trait definition");

pub const PARAMETER_IN_TRAIT_DEFINITION: Rule = Rule::new("parameter in trait definition");

pub const TYPE_IN_TRAIT_DEFINITION: Rule = Rule::new("type in trait definition");

impl Visit for TraitDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.name.range, |visitor, id| {
            let attributes =
                TraitAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let parameters = self
                .parameters
                .as_ref()
                .map(|parameters| parameters.0.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|parameter| {
                    let node = visitor
                        .placeholder_node((id, PARAMETER_IN_TRAIT_DEFINITION), parameter.range);

                    visitor.define_name(
                        &parameter.value,
                        Definition::TypeParameter(TypeParameterDefinition { node }),
                    );

                    node
                })
                .collect::<Vec<_>>();

            visitor.with_target(id, |visitor| {
                self.constraints
                    .r#type
                    .visit(visitor, (id, TYPE_IN_TRAIT_DEFINITION))
            });

            visitor.define_name(
                &self.name.value,
                Definition::Trait(TraitDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters: parameters.clone(),
                    constraints: vec![
                        Constraint::Bound(Bound {
                            tr: id,
                            parameters: parameters.into_iter().map(Ty::Of).collect(),
                        }),
                        // TODO: Other constraints on `self.constraints`
                    ],
                }),
            );

            (PlaceholderNode, TRAIT_DEFINITION)
        })
    }
}
