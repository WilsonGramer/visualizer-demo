use crate::{Definition, Visit, Visitor};
use wipple_compiler_syntax::TraitExpression;
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::{
    constraints::Substitutions,
    nodes::{AnnotateNode, Annotation, EmptyNode, Node},
};

pub static RESOLVED_TRAIT_NAME: Rule = Rule::new("resolved trait name");
pub static UNRESOLVED_TRAIT_NAME: Rule = Rule::new("unresolved trait name");

impl Visit for TraitExpression {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        visitor.typed_node(parent, self.range, |visitor, id| {
            let annotations =
                visitor.resolve_name(&self.r#type.value, id, |definition| match definition {
                    Definition::Type(_) => todo!(),
                    Definition::Trait(definition) => {
                        Some((definition.annotations.clone(), RESOLVED_TRAIT_NAME))
                    }
                    _ => None,
                });

            if let Some((annotations, rule)) = annotations {
                (
                    AnnotateNode {
                        value: id,
                        annotations: vec![Annotation::Instantiate {
                            annotations: annotations.clone(),
                            substitutions: Substitutions::replace_all(),
                        }],
                    }
                    .boxed(),
                    rule,
                )
            } else {
                (EmptyNode.boxed(), UNRESOLVED_TRAIT_NAME)
            }
        })
    }
}
