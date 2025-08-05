use crate::visitor::LazyConstraint;
use wipple_compiler_typecheck::{
    constraints::{Constraint, Ty},
    util::NodeId,
};

pub fn instantiate_constraints(
    node: NodeId,
    constraints: impl IntoIterator<Item = LazyConstraint>,
) -> impl Iterator<Item = Constraint> {
    constraints.into_iter().map(move |lazy| lazy(node))
}

pub fn constraints_for_call(
    function: NodeId,
    inputs: impl IntoIterator<Item = NodeId>,
    output: NodeId,
) -> impl Iterator<Item = Constraint> {
    [Constraint::Ty(
        function,
        Ty::Function {
            inputs: inputs.into_iter().map(Ty::Of).collect(),
            output: Box::new(Ty::Of(output)),
        },
    )]
    .into_iter()
}

pub fn constraints_for_block(
    statements: impl IntoIterator<Item = NodeId>,
    output: NodeId,
) -> impl Iterator<Item = Constraint> {
    let mut constraints = Vec::new();
    for statement in statements {
        constraints.push(Constraint::Ty(statement, Ty::Of(output)));
    }

    if constraints.is_empty() {
        constraints.push(Constraint::Ty(output, Ty::unit()));
    }

    constraints.into_iter()
}

pub fn constraints_for_function(
    function: NodeId,
    inputs: impl IntoIterator<Item = NodeId>,
    output: NodeId,
) -> impl Iterator<Item = Constraint> {
    [Constraint::Ty(
        function,
        Ty::Function {
            inputs: inputs.into_iter().map(Ty::Of).collect(),
            output: Box::new(Ty::Of(output)),
        },
    )]
    .into_iter()
}

pub fn constraints_for_unit(node: NodeId) -> impl Iterator<Item = Constraint> {
    [Constraint::Ty(node, Ty::unit())].into_iter()
}

pub fn constraints_for_tuple(
    node: NodeId,
    elements: impl IntoIterator<Item = NodeId>,
) -> Constraint {
    Constraint::Ty(
        node,
        Ty::Tuple {
            elements: elements.into_iter().map(Ty::Of).collect(),
        },
    )
}
