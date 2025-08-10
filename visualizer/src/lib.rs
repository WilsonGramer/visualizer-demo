mod constraints;
mod graph;
mod groups;
mod solve;

pub use constraints::*;
pub use graph::*;
pub use groups::*;
pub use solve::*;

use std::{fmt::Debug, hash::Hash};

pub trait Db: Sized + 'static {
    type Node: Debug + Copy + Eq + Ord + Hash;

    fn typed_nodes(&self) -> impl Iterator<Item = Self::Node>;

    fn clone_node(&mut self, node: Self::Node, hide: bool) -> Self::Node;

    fn get_trait_instances(
        &mut self,
        source: Self::Node,
        node: Self::Node,
        trait_id: Self::Node,
    ) -> Vec<(Self::Node, Instantiation<Self>)>;

    fn flag_resolved(&mut self, node: Self::Node, instance: Self::Node, ty: Self::Node);
    fn flag_unresolved(&mut self, node: Self::Node, ty: Self::Node);

    fn flag_type(&mut self, node: Self::Node, ty: Ty<Self>);
    fn flag_incomplete_type(&mut self, node: Self::Node);
}
