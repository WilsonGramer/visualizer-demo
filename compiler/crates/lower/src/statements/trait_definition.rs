use crate::{
    Definition, Visit, Visitor,
    attributes::{AttributeParser, TraitAttributes},
};
use wipple_compiler_syntax::TraitDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    nodes::PlaceholderNode,
};

/// A trait definition.
pub const TRAIT_DEFINITION: Rule = Rule::new("trait_definition", &[]);

impl Visit for TraitDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.name.range, |visitor, id| {
            let attributes =
                TraitAttributes::parse(&mut AttributeParser::new(visitor, &self.attributes));

            let ty = self
                .r#type
                .as_ref()
                .map(|ty| visitor.with_target(id, |visitor| ty.visit(visitor, None)));

            visitor.define_name(
                &self.name.source,
                Definition::Trait {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters: self.parameters.clone(),
                    constraints: Vec::from_iter(ty.map(|ty| Constraint::Ty(Ty::Instantiate(ty)))),
                },
            );

            (PlaceholderNode, TRAIT_DEFINITION)
        })
    }
}
