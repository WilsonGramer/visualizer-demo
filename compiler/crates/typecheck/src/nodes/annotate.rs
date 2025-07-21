use crate::{
    constraints::{Bound, ToConstraints, ToConstraintsContext, Ty},
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
    Instantiated(NodeId),
    Instantiate {
        definition: NodeId,
        // `None` is equivalent to substituting all parameters
        substitutions: Option<BTreeMap<NodeId, NodeId>>,
    },
    Type {
        definition: NodeId,
        substitutions: BTreeMap<NodeId, NodeId>,
    },
    Bound {
        node: NodeId,
        tr: NodeId,
        // `None` is equivalent to substituting all parameters
        substitutions: Option<BTreeMap<NodeId, NodeId>>,
    },
}

impl Annotation {
    pub fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        match *self {
            Annotation::Node(other) => {
                ctx.constraints().insert_ty(node, Ty::of(other), ANNOTATED);
            }
            Annotation::Parameter(parameter) => {
                ctx.constraints()
                    .insert_ty(node, Ty::Parameter(parameter), ANNOTATED);
            }
            Annotation::Instantiated(parameter) => {
                ctx.constraints()
                    .insert_ty(node, Ty::Instantiated(parameter), ANNOTATED);
            }
            Annotation::Instantiate {
                definition,
                ref substitutions,
            } => {
                ctx.constraints().insert_ty(
                    node,
                    Ty::Instantiate {
                        instantiation: node,
                        definition,
                        substitutions: substitutions.clone(),
                    },
                    ANNOTATED,
                );
            }
            Annotation::Type {
                definition,
                ref substitutions,
            } => {
                ctx.constraints().insert_ty(
                    node,
                    Ty::Named {
                        name: definition,
                        substitutions: substitutions
                            .iter()
                            .map(|(&parameter, &substitution)| (parameter, Ty::of(substitution)))
                            .collect(),
                    },
                    ANNOTATED_TYPE,
                );
            }
            Annotation::Bound {
                node: bound_node,
                tr,
                ref substitutions,
            } => {
                ctx.constraints().insert_bound(
                    bound_node,
                    Bound {
                        instantiation: node,
                        tr,
                        substitutions: substitutions.clone(),
                    },
                    ANNOTATED_BOUND,
                );
            }
        }
    }
}
