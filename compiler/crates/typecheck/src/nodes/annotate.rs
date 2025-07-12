use crate::{
    constraints::{ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use std::collections::BTreeMap;
use wipple_compiler_trace::{NodeId, Rule};

pub static ANNOTATED: Rule = Rule::new("annotated with value");
pub static ANNOTATED_GENERIC: Rule = Rule::new("annotated with generic value");
pub static ANNOTATED_TYPE: Rule = Rule::new("annotated with type");
pub static ANNOTATED_TRAIT: Rule = Rule::new("annotated with trait");

#[derive(Debug, Clone)]
pub struct AnnotateNode {
    pub value: NodeId,
    pub definition: Annotation,
}

#[derive(Debug, Clone)]
pub enum Annotation {
    Node(NodeId),
    Constant(NodeId),
    Type(NodeId, Vec<NodeId>),
    Trait(NodeId, BTreeMap<NodeId, NodeId>),
}

impl Node for AnnotateNode {}

impl ToConstraints for AnnotateNode {
    fn to_constraints(&self, _node: NodeId, ctx: &ToConstraintsContext<'_>) {
        match self.definition {
            Annotation::Node(node) => {
                ctx.constraints()
                    .insert_ty(self.value, Ty::Of(node), ANNOTATED);
            }
            Annotation::Constant(definition) => {
                ctx.constraints().insert_generic(
                    self.value,
                    definition,
                    BTreeMap::new(),
                    ANNOTATED_GENERIC,
                );
            }
            Annotation::Type(definition, ref parameters) => {
                ctx.constraints().insert_generic(
                    self.value,
                    definition,
                    BTreeMap::new(),
                    ANNOTATED_TYPE,
                );

                ctx.constraints().insert_ty(
                    self.value,
                    Ty::Named {
                        name: definition,
                        parameters: parameters.iter().copied().map(Ty::Of).collect(),
                    },
                    ANNOTATED_GENERIC,
                );
            }
            Annotation::Trait(definition, ref parameters) => {
                ctx.constraints().insert_generic(
                    self.value,
                    definition,
                    parameters.clone(), // TODO: Make a struct with `substitutions` field
                    ANNOTATED_TRAIT,
                );
            }
        }
    }
}
