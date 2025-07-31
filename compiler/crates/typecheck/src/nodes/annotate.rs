use crate::{
    constraints::{
        Bound, Constraint, Instantiation, Substitutions, ToConstraints, ToConstraintsContext, Ty,
    },
    nodes::Node,
};
use wipple_compiler_trace::NodeId;

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

// TODO: Document the difference between annotations and constraints
#[derive(Debug, Clone)]
pub enum Annotation {
    Ty(Ty),
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
            Annotation::Ty(ref ty) => {
                ctx.constraints().push(Constraint::Ty(node, ty.clone()));
            }
            Annotation::Instantiate {
                ref annotations,
                ref substitutions,
            } => {
                let instantiate_ctx = ToConstraintsContext::default();
                for annotation in annotations {
                    annotation.to_constraints(node, &instantiate_ctx);
                }

                ctx.constraints()
                    .push(Constraint::Instantiation(Instantiation {
                        constraints: instantiate_ctx.into_constraints(),
                        substitutions: substitutions.clone(),
                    }));
            }
            Annotation::Bound {
                node: bound_node,
                tr,
                ref substitutions,
            } => {
                ctx.constraints().push(Constraint::Bound(Bound {
                    node: bound_node.unwrap_or(node),
                    tr,
                    substitutions: substitutions.clone(),
                }));
            }
        }
    }
}
