use crate::{
    ConstantDefinition, Definition, Visit, Visitor,
    attributes::{AttributeParser, ConstantAttributes},
};
use wipple_compiler_syntax::ConstantDefinitionStatement;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{AnnotateNode, Annotation};

pub static CONSTANT_DEFINITION: Rule = Rule::new("constant definition");
pub static TYPE_IN_CONSTANT_DEFINITION: Rule = Rule::new("type in constant definition");

impl Visit for ConstantDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let attributes =
                ConstantAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let ty = visitor.with_implicit_type_parameters(|visitor| {
                self.constraints
                    .r#type
                    .visit(visitor, (id, TYPE_IN_CONSTANT_DEFINITION))
            });

            // TODO
            let constraints = self
                .constraints
                .constraints
                .iter()
                .map(|_| todo!())
                .collect::<Vec<_>>();

            visitor.define_name(
                &self.name.value,
                Definition::Constant(ConstantDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    ty,
                    constraints,
                    assigned: false,
                }),
            );

            (
                AnnotateNode {
                    value: id,
                    definition: Annotation::Node(ty),
                },
                CONSTANT_DEFINITION,
            )
        })
    }
}
