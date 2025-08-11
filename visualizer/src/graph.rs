use crate::{Db, Ty, TyGroups};
use itertools::Itertools;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    hash::Hash,
};

pub trait WriteGraphContext<'a> {
    type Node: Debug + Copy + Eq + Ord + Hash;
    type Db: Db<Node = Self::Node>;

    fn format_node(&self, node: Self::Node) -> String;
    fn format_ty(&self, ty: &Ty<Self::Db>) -> String;

    fn include_node(&self, node: Self::Node) -> bool;
    fn related_nodes(&self, node: Self::Node) -> Vec<(Self::Node, String)>;

    fn node_data(&self, node: Self::Node) -> serde_json::Value;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub clusters: Vec<GraphCluster>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphCluster {
    pub id: String,
    pub labels: Vec<String>,
    pub nodes: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

impl Graph {
    pub fn generate<'a, Ctx: WriteGraphContext<'a>>(
        ctx: Ctx,
        ty_groups: &TyGroups<Ctx::Db>,
        nodes: impl IntoIterator<Item = Ctx::Node>,
    ) -> Self {
        let mut visited_groups = BTreeSet::new();
        let mut visited_nodes = BTreeSet::new();
        for node in nodes {
            if !ctx.include_node(node) {
                continue;
            }

            if let Some(group_index) = ty_groups.index_of(node) {
                visited_groups.insert(group_index);
                visited_nodes.insert(node);
            }
        }

        let mut graph_nodes = Vec::new();
        let mut graph_edges = Vec::new();
        let mut visited_relations = BTreeMap::<_, BTreeSet<_>>::new();
        for &node in &visited_nodes {
            graph_nodes.push(GraphNode {
                id: ctx.format_node(node),
                data: ctx.node_data(node),
            });

            for (related, relation) in ctx.related_nodes(node) {
                if !visited_nodes.contains(&related)
                    || visited_relations
                        .get(&(node, related))
                        .is_some_and(|existing| existing.contains(&relation))
                {
                    continue;
                }

                graph_edges.push(GraphEdge {
                    from: ctx.format_node(node),
                    to: ctx.format_node(related),
                    label: relation.clone(),
                });

                visited_relations
                    .entry((node, related))
                    .or_default()
                    .insert(relation);
            }
        }

        let mut graph_clusters = Vec::new();
        for (index, group_tys) in ty_groups.groups() {
            if !visited_groups.contains(&index) {
                continue;
            }

            let nodes_in_group = ty_groups
                .nodes_in_group(index)
                .filter(|node| visited_nodes.contains(node))
                .collect::<Vec<_>>();

            if nodes_in_group.is_empty() {
                continue;
            }

            let labels = group_tys
                .iter()
                .unique()
                .map(|ty| ctx.format_ty(ty))
                .collect::<Vec<_>>();

            let nodes = nodes_in_group
                .into_iter()
                .filter(|&node| ctx.include_node(node))
                .map(|node| ctx.format_node(node))
                .collect::<Vec<_>>();

            graph_clusters.push(GraphCluster {
                id: format!("group{index}"),
                labels,
                nodes,
            });
        }

        Graph {
            nodes: graph_nodes,
            clusters: graph_clusters,
            edges: graph_edges,
        }
    }
}
