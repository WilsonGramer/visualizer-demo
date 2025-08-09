use crate::attributes::{ConstantAttributes, InstanceAttributes, TraitAttributes, TypeAttributes};
use wipple_db::{LazyConstraints, NodeId};
use wipple_syntax::Comments;

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
    pub constraints: LazyConstraints,
    pub value: std::result::Result<NodeId, NodeId>, // Ok(node) or Err(type signature)
}

#[derive(Clone)]
pub struct TypeDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TypeAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: LazyConstraints,
}

#[derive(Clone)]
pub struct TraitDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TraitAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: LazyConstraints,
}

#[derive(Clone)]
pub struct InstanceDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: InstanceAttributes,
    pub tr: NodeId,
    pub value: NodeId,
    // constraints and substitutions are added via facts
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
