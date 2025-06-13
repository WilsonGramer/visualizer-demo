use crate::Context;
use petgraph::Direction;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet, btree_map::Entry};
use wipple_compiler_trace::{AnyRule, NodeId, Rule};
use wipple_compiler_typecheck::constraints::Ty;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    Child {
        r#as: String,

        #[serde(default)]
        rules: Vec<String>,
    },
    Type {
        of: Option<String>,

        r#as: String,

        matches: Option<String>,

        #[serde(default)]
        rules: Vec<String>,
    },
}

impl Selector {
    pub fn run(&self, state: &mut State<'_, '_>) -> Result<(), ()> {
        match self {
            Selector::Child { r#as: name, rules } => state.child(name, rules),
            Selector::Type {
                of,
                r#as: name,
                matches,
                rules,
            } => state.ty(of.as_deref(), name, matches.as_deref(), rules),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTerm {
    pub node: NodeId,
    pub rule: AnyRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TyTerm {
    pub node: NodeId,
    pub ty: Ty,
    pub related: BTreeMap<NodeId, Option<AnyRule>>,
}

#[derive(Clone)]
pub struct State<'ctx, 'a> {
    pub ctx: &'a Context<'ctx>,
    pub root: NodeTerm,
    pub nodes: BTreeMap<String, NodeTerm>,
    pub tys: BTreeMap<String, TyTerm>,
    pub visited_nodes: HashSet<NodeId>,
    pub visited_tys: HashSet<Ty>,
    pub node_progress: bool,
    pub ty_progress: bool,
}

impl<'ctx, 'a> State<'ctx, 'a> {
    pub fn new(ctx: &'a Context<'ctx>, name: Option<String>, term: NodeTerm) -> Self {
        State {
            ctx,
            root: term.clone(),
            nodes: name
                .map(|name| BTreeMap::from([(name, term)]))
                .unwrap_or_default(),
            tys: Default::default(),
            visited_nodes: Default::default(),
            visited_tys: Default::default(),
            node_progress: false,
            ty_progress: false,
        }
    }

    pub fn child(&mut self, child: &str, rules: &[String]) -> Result<(), ()> {
        let matches_rules =
            |rule: &AnyRule| rules.is_empty() || rules.iter().any(|r| rule.name() == r);

        match self.nodes.entry(child.to_string()) {
            Entry::Vacant(entry) => {
                let term = self
                    .ctx
                    .relations
                    .edges(self.root.node)
                    .find_map(|(parent, child, child_rule)| {
                        if parent == self.root.node
                            && !self.visited_nodes.contains(&child)
                            && matches_rules(child_rule)
                        {
                            Some(NodeTerm {
                                node: child,
                                rule: *child_rule,
                            })
                        } else {
                            None
                        }
                    })
                    .ok_or(())?;

                entry.insert(term.clone());
                self.visited_nodes.insert(term.node);
                self.node_progress = true;

                Ok(())
            }
            Entry::Occupied(entry) => {
                if matches_rules(&entry.get().rule) {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn ty(
        &mut self,
        node: Option<&str>,
        name: &str,
        matches: Option<&str>,
        rules: &[String],
    ) -> Result<(), ()> {
        let matches_rules = |ty_rules: &BTreeMap<NodeId, Option<AnyRule>>| {
            rules.is_empty()
                || rules.iter().any(|r| {
                    ty_rules
                        .values()
                        .flatten()
                        .any(|ty_rule| ty_rule.name() == r)
                })
        };

        match self.tys.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let term = node
                    .and_then(|node| self.nodes.get(node))
                    .unwrap_or(&self.root);

                let tys = self.ctx.tys.get(&term.node).ok_or(())?;

                let (ty, group_id) = tys
                    .iter()
                    .find(|(ty, _)| !self.visited_tys.contains(ty))
                    .cloned()
                    .ok_or(())?;

                // FIXME: This doesn't work with placeholders; the type should
                // actually be parsed
                if matches.is_some_and(|filter| filter != ty.to_debug_string(self.ctx.feedback)) {
                    return Err(());
                }

                let related = group_id
                    .map(|id| self.ctx.groups.get(&id).unwrap().clone())
                    .unwrap_or_default();

                if !matches_rules(&related) {
                    return Err(());
                }

                let term = TyTerm {
                    node: term.node,
                    ty: ty.clone(),
                    related,
                };

                entry.insert(term.clone());
                self.visited_tys.insert(ty);
                self.ty_progress = true;

                Ok(())
            }
            Entry::Occupied(entry) => {
                let term = entry.get();

                if matches_rules(&term.related) {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
}
