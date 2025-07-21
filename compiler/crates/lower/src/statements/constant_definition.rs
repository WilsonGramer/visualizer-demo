use crate::{
    ConstantDefinition, Definition, Visit, Visitor,
    attributes::{AttributeParser, ConstantAttributes},
};
use std::mem;
use wipple_compiler_syntax::{ConstantDefinitionStatement, Constraints};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::nodes::{Annotation, EmptyNode};

pub static CONSTANT_DEFINITION: Rule = Rule::new("constant definition");
pub static TYPE_IN_CONSTANT_DEFINITION: Rule = Rule::new("type in constant definition");
pub static CONSTRAINT_IN_CONSTANT_DEFINITION: Rule = Rule::new("constraint in constant definition");

impl Visit for ConstantDefinitionStatement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.node(parent, self.range, |visitor, id| {
            let attributes =
                ConstantAttributes::parse(&mut AttributeParser::new(id, visitor, &self.attributes));

            let (ty, annotations) = visitor.with_definition(|visitor| {
                visitor.current_definition().implicit_type_parameters = true;

                let ty = self
                    .constraints
                    .r#type
                    .visit(visitor, (id, TYPE_IN_CONSTANT_DEFINITION));

                visitor
                    .current_definition()
                    .annotations
                    .push(Annotation::Instantiate {
                        definition: ty,
                        substitutions: None,
                    });

                visitor.current_definition().instantiate_type_parameters = true;

                if let Some(Constraints(constraints)) = &self.constraints.constraints {
                    for constraint in constraints {
                        constraint.visit(visitor, (id, CONSTRAINT_IN_CONSTANT_DEFINITION));
                    }
                }

                (ty, mem::take(&mut visitor.current_definition().annotations))
            });

            visitor.define_name(
                &self.name.value,
                Definition::Constant(ConstantDefinition {
                    node: id,
                    comments: self.comments.clone(),
                    attributes,
                    annotations,
                    value: Err(ty),
                }),
            );

            (EmptyNode, CONSTANT_DEFINITION)
        })
    }
}
