use crate::{
    constraints::{Constraint, ToConstraints, ToConstraintsContext, Ty},
    nodes::Node,
};
use wipple_compiler_trace::{NodeId, rule};

#[derive(Debug, Clone)]
pub struct DefinitionNode {
    pub definition: NodeId,
    pub constraints: Vec<Constraint>,
}

impl Node for DefinitionNode {}

rule! {
    /// Constraints are inherited from the definition.
    definition: Extra;
}

impl ToConstraints for DefinitionNode {
    fn to_constraints(&self, node: NodeId, ctx: &ToConstraintsContext<'_>) {
        ctx.constraints()
            .insert_ty(node, Ty::Of(self.definition), rule::definition);

        ctx.constraints()
            .extend(node, self.constraints.clone(), rule::definition);
    }
}
