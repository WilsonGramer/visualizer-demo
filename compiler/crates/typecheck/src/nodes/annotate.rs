use crate::{
    constraints::{Bound, Instantiation, Substitutions, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use std::collections::BTreeMap;
use wipple_compiler_trace::{NodeId, Rule};

pub static ANNOTATED: Rule = Rule::new("annotated with value");
pub static ANNOTATED_TYPE: Rule = Rule::new("annotated with type");
pub static ANNOTATED_BOUND: Rule = Rule::new("annotated with bound");

#[derive(Debug, Clone)]
pub struct AnnotateNode {
    pub value: NodeId,
    pub annotations: Vec<Annotation>,
}

impl Node for AnnotateNode {}

impl ToConstraints for AnnotateNode {
    fn to_constraints(&self, _node: NodeId, ctx: &ToConstraintsContext<'_>) {
        for annotation in &self.annotations {
            annotation.to_constraints(self.value, ctx);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Annotation {
    Node(NodeId),
    Parameter(NodeId),
    Type {
        definition: NodeId,
        parameters: BTreeMap<NodeId, NodeId>,
    },
    Instantiate {
        annotations: Vec<Annotation>,
        substitutions: Substitutions,
    },
    Bound {
        node: Option<NodeId>,
        tr: NodeId,
        substitutions: Substitutions,
    },
}

impl Annotation {
    pub fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        match *self {
            Annotation::Node(other) => {
                ctx.constraints().insert_ty(node, Ty::Of(other), ANNOTATED);
            }
            Annotation::Parameter(parameter) => {
                ctx.constraints()
                    .insert_ty(node, Ty::Parameter(parameter), ANNOTATED);
            }
            Annotation::Type {
                definition,
                ref parameters,
            } => {
                ctx.constraints().insert_ty(
                    node,
                    Ty::Named {
                        name: definition,
                        parameters: parameters
                            .iter()
                            .map(|(&parameter, &ty)| (parameter, Ty::Of(ty)))
                            .collect(),
                    },
                    ANNOTATED_TYPE,
                );
            }
            Annotation::Instantiate {
                ref annotations,
                ref substitutions,
            } => {
                let instantiate_ctx = ToConstraintsContext::default();
                for annotation in annotations {
                    annotation.to_constraints(node, &instantiate_ctx);
                }

                ctx.constraints().insert_instantiation(Instantiation {
                    constraints: instantiate_ctx.into_constraints().iter().collect(), // TODO: Just use `Vec<Constraint>` instead of `Constraints` wrapper type
                    substitutions: substitutions.clone(),
                });
            }
            Annotation::Bound {
                node: bound_node,
                tr,
                ref substitutions,
            } => {
                ctx.constraints().insert_bound(
                    Bound {
                        node: bound_node.unwrap_or(node),
                        tr,
                        substitutions: substitutions.clone(),
                    },
                    ANNOTATED_BOUND,
                );
            }
        }
    }
}
