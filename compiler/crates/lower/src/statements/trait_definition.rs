use crate::{
    Definition, TraitDefinition, TypeParameterDefinition, Visit, Visitor,
    attributes::{AttributeParser, TraitAttributes},
};
use wipple_compiler_syntax::TraitDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Bound, Constraint, Ty},
    nodes::{AnnotateNode, Annotation},
};

pub static TRAIT_DEFINITION: Rule = Rule::new("trait definition");
pub static PARAMETER_IN_TRAIT_DEFINITION: Rule = Rule::new("parameter in trait definition");
pub static TYPE_IN_TRAIT_DEFINITION: Rule = Rule::new("type in trait definition");

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

            let ty = self
                .constraints
                .r#type
                .visit(visitor, (id, TYPE_IN_TRAIT_DEFINITION));

            visitor.define_name(
                &self.name.value,
                Definition::Trait(TraitDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters: parameters.clone(),
                    constraints: vec![
                        Constraint::Bound(
                            Bound {
                                tr: id,
                                parameters: parameters.into_iter().map(Ty::Of).collect(),
                            },
                            TRAIT_DEFINITION,
                        ),
                        // TODO: Other constraints on `self.constraints`
                    ],
                }),
            );

            (
                AnnotateNode {
                    value: id,
                    definition: Annotation::Node(ty),
                },
                TRAIT_DEFINITION,
            )
        })
    }
}
