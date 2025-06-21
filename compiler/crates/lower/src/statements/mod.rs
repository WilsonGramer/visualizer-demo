mod assignment;
mod constant_definition;
mod expression;
mod instance_definition;
mod trait_definition;
mod type_definition;

use crate::{Visit, Visitor};
use wipple_compiler_syntax::Statement;
use wipple_compiler_trace::{NodeId, Rule};

impl Visit for Statement {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId {
        match self {
            Statement::ConstantDefinition(statement) => statement.visit(visitor, parent),
            Statement::Expression(statement) => statement.visit(visitor, parent),
            Statement::Assignment(statement) => statement.visit(visitor, parent),
            Statement::TypeDefinition(statement) => statement.visit(visitor, parent),
            Statement::TraitDefinition(statement) => statement.visit(visitor, parent),
            Statement::InstanceDefinition(statement) => statement.visit(visitor, parent),
        }
    }
}
