use crate::{
    attributes::{ConstantAttributes, InstanceAttributes, TraitAttributes, TypeAttributes},
    visitor::LazyConstraint,
};
use std::collections::BTreeMap;
use wipple_compiler_syntax::Comments;
use wipple_compiler_trace::NodeId;

#[derive(Clone)]
pub enum Definition {
    Variable(VariableDefinition),
    Constant(ConstantDefinition),
    Type(TypeDefinition),
    Trait(TraitDefinition),
    Instance(InstanceDefinition),
    TypeParameter(TypeParameterDefinition),
}

#[derive(Clone)]
pub struct VariableDefinition {
    pub node: NodeId,
}

#[derive(Clone)]
pub struct ConstantDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: ConstantAttributes,
    pub constraints: Vec<LazyConstraint>,
    pub value: std::result::Result<NodeId, NodeId>, // Ok(node) or Err(type signature)
}

#[derive(Clone)]
pub struct TypeDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TypeAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<LazyConstraint>,
}

#[derive(Clone)]
pub struct TraitDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TraitAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<LazyConstraint>,
}

#[derive(Clone)]
pub struct InstanceDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: InstanceAttributes,
    pub tr: NodeId,
    pub substitutions: BTreeMap<NodeId, NodeId>,
    pub constraints: Vec<LazyConstraint>,
    pub value: NodeId,
}

#[derive(Clone)]
pub struct TypeParameterDefinition {
    pub node: NodeId,
}

impl Definition {
    pub fn source(&self) -> NodeId {
        match self {
            Definition::Variable(definition) => definition.node,
            Definition::Constant(definition) => definition.node,
            Definition::Type(definition) => definition.node,
            Definition::Trait(definition) => definition.node,
            Definition::Instance(definition) => definition.node,
            Definition::TypeParameter(definition) => definition.node,
        }
    }

    pub fn comments(&self) -> Option<&Comments> {
        match self {
            Definition::Variable(_) => None,
            Definition::Constant(definition) => Some(&definition.comments),
            Definition::Type(definition) => Some(&definition.comments),
            Definition::Trait(definition) => Some(&definition.comments),
            Definition::Instance(definition) => Some(&definition.comments),
            Definition::TypeParameter(_) => None,
        }
    }
}
