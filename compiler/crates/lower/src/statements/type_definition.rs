use crate::{
    Definition, Visit, Visitor,
    attributes::{AttributeParser, TypeAttributes},
};
use wipple_compiler_syntax::TypeDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule, rule};
use wipple_compiler_typecheck::nodes::PlaceholderNode;

rule! {
    /// A type definition.
    type_definition: Extra;
}

impl Visit for TypeDefinitionStatement {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId {
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

            (PlaceholderNode, rule::type_definition)
        })
    }
}
