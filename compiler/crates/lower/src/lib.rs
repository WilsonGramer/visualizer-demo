macro_rules! nodes {
    ($(mod $mod:ident;)*) => {
        $(mod $mod;)*

        pub mod rule {
            $(pub use super::$mod::rule::*;)*
        }
    };
}

use nodes;

nodes! {
    mod attributes;
    mod expressions;
    mod patterns;
    mod statements;
    mod tys;
}

use crate::attributes::{ConstantAttributes, InstanceAttributes, TraitAttributes, TypeAttributes};
use petgraph::prelude::DiGraphMap;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
};
use wipple_compiler_syntax::{Comment, SourceFile, TypeParameterName};
use wipple_compiler_trace::{AnyRule, NodeId, Rule, Span};
use wipple_compiler_typecheck::{
    constraints::{Bound, Constraint},
    nodes::{Node, PlaceholderNode},
};

#[derive(Debug)]
pub struct Result {
    pub nodes: BTreeMap<NodeId, (Box<dyn Node>, AnyRule)>,
    pub spans: BTreeMap<NodeId, Span>,
    pub names: HashMap<String, NodeId>,
    pub relations: DiGraphMap<NodeId, AnyRule>,
}

pub fn visit(file: &SourceFile, make_span: impl Fn(Range<usize>) -> Span) -> Result {
    let mut visitor = Visitor::new(make_span);

    for statement in &file.statements {
        statement.visit_root(&mut visitor);
    }

    Result {
        nodes: visitor
            .nodes
            .into_iter()
            .map(|(id, node)| (id, node.unwrap()))
            .collect(),
        spans: visitor.spans,
        names: visitor
            .scopes
            .pop()
            .unwrap()
            .0
            .into_iter()
            .map(|(name, definition)| (name, definition.source()))
            .collect(),
        relations: visitor.relations,
    }
}

trait Visit {
    fn visit<'a>(
        &'a self,
        visitor: &mut Visitor<'a>,
        parent: Option<(NodeId, impl Rule)>,
    ) -> NodeId;

    fn visit_root<'a>(&'a self, visitor: &mut Visitor<'a>) -> NodeId {
        self.visit(visitor, None::<(_, AnyRule)>)
    }
}

struct Visitor<'a> {
    make_span: Box<dyn Fn(Range<usize>) -> Span + 'a>,
    nodes: BTreeMap<NodeId, Option<(Box<dyn Node>, AnyRule)>>,
    spans: BTreeMap<NodeId, Span>,
    relations: DiGraphMap<NodeId, AnyRule>,
    stack: Vec<NodeId>, // used by patterns
    scopes: Vec<Scope>, // used by blocks
}

impl<'a> Visitor<'a> {
    fn new(make_span: impl Fn(Range<usize>) -> Span + 'a) -> Self {
        Visitor {
            make_span: Box::new(make_span),
            nodes: Default::default(),
            spans: Default::default(),
            relations: Default::default(),
            stack: Default::default(),
            scopes: vec![Scope::default()],
        }
    }

    fn node<N: Node, R: Rule>(
        &mut self,
        parent: Option<(NodeId, impl Rule)>,
        range: &Range<usize>,
        f: impl FnOnce(&mut Self, NodeId) -> (N, R),
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.insert(id, None);

        if let Some((parent, rule)) = parent {
            self.relations.add_edge(parent, id, rule.erased());
            self.stack.push(parent);
        }

        let (node, rule) = f(self, id);

        if parent.is_some() {
            self.stack.pop();
        }

        self.nodes.insert(
            id,
            // Use `boxed` instead of `Box::new` in case `node` is already a `Box`
            Some((node.boxed(), rule.erased())),
        );

        self.spans.insert(id, (self.make_span)(range.clone()));

        id
    }

    fn root_node<N: Node, R: Rule>(
        &mut self,
        range: &Range<usize>,
        f: impl FnOnce(&mut Self, NodeId) -> (N, R),
    ) -> NodeId {
        self.node(None::<(_, AnyRule)>, range, f)
    }

    fn root_placeholder_node<R: Rule>(&mut self, range: &Range<usize>, rule: R) -> NodeId {
        self.root_node(range, |_, _| (PlaceholderNode, rule))
    }

    fn parent(&self) -> NodeId {
        *self.stack.last().expect("no parent")
    }
}

#[derive(Debug, Clone, Default)]
struct Scope(HashMap<String, Definition>);

#[derive(Debug, Clone)]
enum Definition {
    Variable {
        node: NodeId,
    },
    Constant {
        node: NodeId,
        comments: Vec<Comment>,
        attributes: ConstantAttributes,
        constraints: Vec<Constraint>,
    },
    Type {
        node: NodeId,
        comments: Vec<Comment>,
        attributes: TypeAttributes,
        parameters: Vec<TypeParameterName>,
        // TODO: representation
    },
    Trait {
        node: NodeId,
        comments: Vec<Comment>,
        attributes: TraitAttributes,
        parameters: Vec<TypeParameterName>,
        constraints: Vec<Constraint>,
        // TODO: value
    },
    Instance {
        node: NodeId,
        comments: Vec<Comment>,
        attributes: InstanceAttributes,
        bound: Bound,
        constraints: Vec<Constraint>,
    },
}

impl Definition {
    fn source(&self) -> NodeId {
        match self {
            Definition::Variable { node, .. }
            | Definition::Constant { node, .. }
            | Definition::Type { node, .. }
            | Definition::Trait { node, .. }
            | Definition::Instance { node, .. } => *node,
        }
    }
}

impl Visitor<'_> {
    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_name(&mut self, name: &str, node: NodeId, rule: impl Rule) -> Option<&Definition> {
        let definition = self.scopes.iter().rev().find_map(|scope| scope.0.get(name));

        if let Some(definition) = definition {
            self.relations
                .add_edge(definition.source(), node, rule.erased());
        }

        definition
    }

    fn peek_name(&mut self, name: &str) -> Option<&Definition> {
        self.scopes.iter().rev().find_map(|scope| scope.0.get(name))
    }

    fn define_name(&mut self, name: &str, definition: Definition) {
        self.scopes
            .last_mut()
            .unwrap()
            .0
            .insert(name.to_string(), definition);
    }
}
