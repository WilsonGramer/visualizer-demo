use crate::Ty;
use derive_where::derive_where;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

#[derive_where(Debug, Clone, Default)]
pub struct TyGroups<Db: crate::Db> {
    indices: BTreeMap<Db::Node, u32>,
    tys: Vec<Vec<Ty<Db>>>,
}

impl<Db: crate::Db> TyGroups<Db> {
    pub fn insert_group(&mut self, ty: Ty<Db>) -> u32 {
        let index = self.tys.len() as u32;
        self.tys.push(vec![ty]);
        index
    }

    pub fn assign_node_to_index(&mut self, node: Db::Node, index: u32) {
        self.indices.insert(node, index);
    }

    pub fn index_of(&self, node: Db::Node) -> Option<u32> {
        self.indices.get(&node).copied()
    }

    pub fn tys_at(&self, index: u32) -> &[Ty<Db>] {
        &self.tys[index as usize]
    }

    pub fn tys_at_mut(&mut self, index: u32) -> &mut Vec<Ty<Db>> {
        &mut self.tys[index as usize]
    }

    pub fn nodes_in_group(&self, index: u32) -> impl Iterator<Item = Db::Node> {
        self.indices
            .iter()
            .filter_map(move |(&node, &i)| (i == index).then_some(node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = Db::Node> {
        self.indices.keys().copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = (u32, &[Ty<Db>])> {
        self.tys
            .iter()
            .enumerate()
            .map(|(index, tys)| (index as u32, tys.as_slice()))
    }
}

#[derive_where(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupKey<Db: crate::Db>(u32, PhantomData<Db>);

#[derive_where(Debug, Clone)]
pub struct Group<Db: crate::Db>(pub BTreeSet<Db::Node>);

impl<Db: crate::Db> Group<Db> {
    pub fn new(node: Db::Node) -> Self {
        Group(BTreeSet::from([node]))
    }
}

impl<Db: crate::Db> ena::unify::UnifyKey for GroupKey<Db> {
    type Value = Group<Db>;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(index: u32) -> Self {
        GroupKey(index, PhantomData)
    }

    fn tag() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<Db: crate::Db> ena::unify::UnifyValue for Group<Db> {
    type Error = std::convert::Infallible;

    fn unify_values(left: &Self, right: &Self) -> Result<Self, Self::Error> {
        Ok(Group(left.0.union(&right.0).copied().collect()))
    }
}

#[derive_where(Debug, Clone, Default)]
pub struct GroupKeys<Db: crate::Db> {
    keys: BTreeMap<Db::Node, GroupKey<Db>>,
    nodes: BTreeMap<GroupKey<Db>, Db::Node>,
}

impl<Db: crate::Db> GroupKeys<Db> {
    pub fn key_for_node(
        &mut self,
        node: Db::Node,
        init: impl FnOnce() -> GroupKey<Db>,
    ) -> GroupKey<Db> {
        *self.keys.entry(node).or_insert_with(|| {
            let key = init();
            self.nodes.insert(key, node);
            key
        })
    }

    pub fn try_key_for_node(&self, node: Db::Node) -> Option<GroupKey<Db>> {
        self.keys.get(&node).copied()
    }

    pub fn update_representative(&mut self, node: Db::Node, group: GroupKey<Db>) {
        self.keys.insert(node, group);
        self.nodes.insert(group, node);
    }

    pub fn node_for_key(&self, key: GroupKey<Db>) -> Db::Node {
        self.try_node_for_key(key).unwrap()
    }

    fn try_node_for_key(&self, key: GroupKey<Db>) -> Option<Db::Node> {
        self.nodes.get(&key).copied()
    }

    pub fn nodes(&self) -> impl Iterator<Item = Db::Node> {
        self.keys.keys().copied()
    }
}
