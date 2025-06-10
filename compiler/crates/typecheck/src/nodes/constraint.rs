use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, rule};

#[derive(Debug, Clone)]
pub struct ConstraintNode {
    pub value: NodeId,
    pub constraints: Vec<Constraint>,
}

impl Node for ConstraintNode {}

rule! {
    /// Constraints are inherited from explicit annotations.
    annotated: Extra;
}

impl ToConstraints for ConstraintNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints()
            .extend(self.value, self.constraints.clone(), rule::annotated);

        ctx.constraints().extend(
            node,
            vec![Constraint::Ty(Ty::influences(self.value))],
            rule::annotated,
        );
    }
}
