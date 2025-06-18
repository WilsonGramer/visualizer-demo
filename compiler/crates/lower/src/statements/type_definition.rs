use crate::{
    Definition, Visit, Visitor,
    attributes::{AttributeParser, TypeAttributes},
};
use wipple_compiler_syntax::TypeDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

/// A type definition.
pub const TYPE_DEFINITION: Rule = Rule::new("type_definition", &[]);

impl Visit for TypeDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: Option<(NodeId, Rule)>) -> NodeId {
        visitor.node(parent, &self.name.range, |visitor, id| {
            if self.representation.is_some() {
                todo!();
            }

            let attributes =
                TypeAttributes::parse(&mut AttributeParser::new(visitor, &self.attributes));

            visitor.define_name(
                &self.name.source,
                Definition::Type {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    parameters: self.parameters.clone(),
                },
            );

            (PlaceholderNode, TYPE_DEFINITION)
        })
    }
}
