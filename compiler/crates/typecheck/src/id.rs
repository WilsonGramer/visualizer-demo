use std::fmt::Debug;
use wipple_compiler_trace::NodeId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct TypedNodeId {
    id: NodeId,

    // Generic definitions contain nodes that all usages will reference. This
    // tracks which specific node instantiated the definition, so constraints
    // aren't shared when `self` is compared to other node IDs.
    pub instantiation: Option<NodeId>,
}

impl From<NodeId> for TypedNodeId {
    fn from(id: NodeId) -> Self {
        TypedNodeId {
            id,
            instantiation: None,
        }
    }
}

impl TypedNodeId {
    pub fn instantiate(id: NodeId, instantiation: NodeId) -> Self {
        TypedNodeId {
            id,
            instantiation: (id != instantiation).then_some(instantiation),
        }
    }

    pub fn instantiated_under(mut self, instantiation: NodeId) -> Self {
        assert!(
            self.instantiation.is_none_or(|i| i == instantiation),
            "already instantiated"
        );

        if self.id != instantiation {
            self.instantiation = Some(instantiation);
        } else {
            // This node is the instantiation
        }

        self
    }

    pub fn uninstantiated(mut self) -> Self {
        self.instantiation = None;
        self
    }

    pub fn untyped(self) -> NodeId {
        self.id
    }
}

impl Debug for TypedNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.instantiation {
            Some(instantiation) => {
                write!(f, "{:?} instantiated under {:?}", self.id, instantiation)
            }
            None => write!(f, "{:?}", self.id),
        }
    }
}
