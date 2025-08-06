use crate::Ty;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};
use visualizer_db::NodeId;

#[derive(Debug, Clone, Default)]
pub struct TyGroups {
    indices: BTreeMap<NodeId, u32>,
    tys: Vec<Vec<Ty>>,
}

impl TyGroups {
    pub fn insert_group(&mut self, ty: Ty) -> u32 {
        let index = self.tys.len() as u32;
        self.tys.push(vec![ty]);
        index
    }

    pub fn assign_node_to_index(&mut self, node: NodeId, index: u32) {
        self.indices.insert(node, index);
    }

    pub fn index_of(&self, node: NodeId) -> Option<u32> {
        self.indices.get(&node).copied()
    }

    pub fn tys_at(&self, index: u32) -> &[Ty] {
        &self.tys[index as usize]
    }

    pub fn tys_at_mut(&mut self, index: u32) -> &mut Vec<Ty> {
        &mut self.tys[index as usize]
    }

    pub fn nodes_in_group(&self, index: u32) -> impl Iterator<Item = NodeId> {
        self.indices
            .iter()
            .filter_map(move |(&node, &i)| (i == index).then_some(node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> {
        self.indices.keys().copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = (u32, &[Ty])> {
        self.tys
            .iter()
            .enumerate()
            .map(|(index, tys)| (index as u32, tys.as_slice()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupKey(u32);

#[derive(Debug, Clone)]
pub struct Group(pub BTreeSet<NodeId>);

impl Group {
    pub fn new(node: NodeId) -> Self {
        Group(BTreeSet::from([node]))
    }
}

impl ena::unify::UnifyKey for GroupKey {
    type Value = Group;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(index: u32) -> Self {
        GroupKey(index)
    }

    fn tag() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl ena::unify::UnifyValue for Group {
    type Error = std::convert::Infallible;

    fn unify_values(left: &Self, right: &Self) -> Result<Self, Self::Error> {
        Ok(Group(left.0.union(&right.0).copied().collect()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroupKeys {
    keys: BTreeMap<NodeId, GroupKey>,
    nodes: BTreeMap<GroupKey, NodeId>,
}

impl GroupKeys {
    pub fn key_for_node(&mut self, node: NodeId, init: impl FnOnce() -> GroupKey) -> GroupKey {
        *self.keys.entry(node).or_insert_with(|| {
            let key = init();
            self.nodes.insert(key, node);
            key
        })
    }

    pub fn try_key_for_node(&self, node: NodeId) -> Option<GroupKey> {
        self.keys.get(&node).copied()
    }

    pub fn update_representative(&mut self, node: NodeId, group: GroupKey) {
        self.keys.insert(node, group);
        self.nodes.insert(group, node);
    }

    pub fn node_for_key(&self, key: GroupKey) -> NodeId {
        self.try_node_for_key(key).unwrap()
    }

    fn try_node_for_key(&self, key: GroupKey) -> Option<NodeId> {
        self.nodes.get(&key).copied()
    }
}
