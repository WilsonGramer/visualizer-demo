mod constraints;
mod groups;
mod solve;
mod visualize;

pub use constraints::*;
pub use groups::*;
pub use solve::*;
pub use visualize::*;

use std::{fmt::Debug, hash::Hash};

pub trait Db: Sized + 'static {
    type Node: Debug + Copy + Eq + Ord + Hash;

    fn typed_nodes(&self) -> impl Iterator<Item = Self::Node>;

    fn clone_node(&mut self, node: Self::Node) -> Self::Node;

    fn get_trait_instances(
        &mut self,
        trait_id: Self::Node,
    ) -> Vec<(Self::Node, Substitutions<Self>)>;

    fn flag_resolved(&mut self, node: Self::Node, bound: Bound<Self>, instance: Self::Node);
    fn flag_unresolved(&mut self, node: Self::Node, bound: Bound<Self>);

    fn flag_type(&mut self, node: Self::Node, ty: Ty<Self>);
    fn flag_incomplete_type(&mut self, node: Self::Node);
    fn flag_unknown_type(&mut self, node: Self::Node);
}
