mod attributes;
mod expressions;
mod patterns;
mod statements;
mod tys;

use crate::attributes::{ConstantAttributes, InstanceAttributes, TraitAttributes, TypeAttributes};
use petgraph::prelude::DiGraphMap;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use wipple_compiler_syntax::{Comments, Range, SourceFile, Statement};
use wipple_compiler_trace::{NodeId, Rule, Span};
use wipple_compiler_typecheck::{
    constraints::Constraint,
    nodes::{EmptyNode, Node},
};

pub static SOURCE_FILE: Rule = Rule::new("source file");
pub static STATEMENT_IN_SOURCE_FILE: Rule = Rule::new("statement in source file");

#[derive(Debug)]
pub struct Result {
    pub nodes: BTreeMap<NodeId, (Box<dyn Node>, Rule)>,
    pub typed_nodes: BTreeSet<NodeId>,
    pub spans: BTreeMap<NodeId, Span>,
    pub relations: DiGraphMap<NodeId, Rule>,
    pub definitions: BTreeMap<NodeId, Definition>,
    pub instances: BTreeMap<NodeId, Vec<NodeId>>,
}

pub fn visit(file: &SourceFile, make_span: impl Fn(Range) -> Span) -> Result {
    let mut visitor = Visitor::new(make_span);

    let source_file = visitor.node_inner(None, file.range, |visitor, id| {
        if let Some(statements) = &file.statements {
            for statement in &statements.0 {
                if !matches!(statement, Statement::Empty(_)) {
                    statement.visit(visitor, (id, STATEMENT_IN_SOURCE_FILE));
                }
            }
        }

        (EmptyNode, SOURCE_FILE)
    });

    visitor.nodes.remove(&source_file);
    visitor.relations.remove_node(source_file);

    let nodes = visitor
        .nodes
        .into_iter()
        .filter_map(|(id, node)| Some((id, node?)))
        .collect();

    let mut definitions = visitor
        .scopes
        .pop()
        .unwrap()
        .definitions
        .into_values()
        .flatten()
        .map(|definition| (definition.source(), definition))
        .collect::<BTreeMap<_, _>>();

    let mut instance_ids = BTreeMap::<_, Vec<_>>::new();
    for (tr, instances) in visitor.instances {
        for instance in instances {
            definitions.insert(instance.node, Definition::Instance(instance.clone()));

            instance_ids.entry(tr).or_default().push(NodeId {
                namespace: None,
                index: instance.node.index,
            });
        }
    }

    Result {
        nodes,
        typed_nodes: visitor.typed_nodes,
        spans: visitor.spans,
        relations: visitor.relations,
        definitions,
        instances: instance_ids,
    }
}

trait Visit {
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>, parent: (NodeId, Rule)) -> NodeId;
}

struct Visitor<'a> {
    make_span: Box<dyn Fn(Range) -> Span + 'a>,
    nodes: BTreeMap<NodeId, Option<(Box<dyn Node>, Rule)>>,
    typed_nodes: BTreeSet<NodeId>,
    spans: BTreeMap<NodeId, Span>,
    relations: DiGraphMap<NodeId, Rule>,
    scopes: Vec<Scope>,
    instances: BTreeMap<NodeId, Vec<InstanceDefinition>>,
    implicit_type_parameters: bool,
}

impl<'a> Visitor<'a> {
    fn new(make_span: impl Fn(Range) -> Span + 'a) -> Self {
        Visitor {
            make_span: Box::new(make_span),
            nodes: Default::default(),
            typed_nodes: Default::default(),
            spans: Default::default(),
            relations: Default::default(),
            scopes: vec![Scope::default()],
            instances: Default::default(),
            implicit_type_parameters: false,
        }
    }

    fn node<N: Node>(
        &mut self,
        parent: (NodeId, Rule),
        range: Range,
        f: impl FnOnce(&mut Self, NodeId) -> (N, Rule),
    ) -> NodeId {
        self.node_inner(Some(parent), range, f)
    }

    fn typed_node<N: Node>(
        &mut self,
        parent: (NodeId, Rule),
        range: Range,
        f: impl FnOnce(&mut Self, NodeId) -> (N, Rule),
    ) -> NodeId {
        let id = self.node_inner(Some(parent), range, f);
        self.typed_nodes.insert(id);
        id
    }

    fn node_inner<N: Node>(
        &mut self,
        parent: Option<(NodeId, Rule)>,
        range: Range,
        f: impl FnOnce(&mut Self, NodeId) -> (N, Rule),
    ) -> NodeId {
        let id = NodeId {
            namespace: None,
            index: self.nodes.len() as u32,
        };

        self.nodes.insert(id, None);

        if let Some((parent, rule)) = parent {
            self.relations.add_edge(parent, id, rule);
        }

        let (node, rule) = f(self, id);

        self.nodes.insert(
            id,
            // Use `boxed` instead of `Box::new` in case `node` is already a `Box`
            Some((node.boxed(), rule)),
        );

        self.spans.insert(id, (self.make_span)(range));

        id
    }

    fn placeholder_node(&mut self, parent: (NodeId, Rule), range: Range) -> NodeId {
        let (parent_id, parent_rule) = parent;

        self.node((parent_id, parent_rule), range, |_, _| {
            (EmptyNode, parent_rule)
        })
    }
}

#[derive(Debug, Clone, Default)]
struct Scope {
    definitions: HashMap<String, Vec<Definition>>,
}

#[derive(Debug, Clone)]
pub enum Definition {
    Variable(VariableDefinition),
    Constant(ConstantDefinition),
    Type(TypeDefinition),
    Trait(TraitDefinition),
    Instance(InstanceDefinition),
    TypeParameter(TypeParameterDefinition),
}

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub node: NodeId,
}

#[derive(Debug, Clone)]
pub struct ConstantDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: ConstantAttributes,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TypeAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: TraitAttributes,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct InstanceDefinition {
    pub node: NodeId,
    pub comments: Comments,
    pub attributes: InstanceAttributes,
    pub tr: NodeId,
    pub parameters: Vec<NodeId>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
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

    pub fn constraints(&self) -> Vec<Constraint> {
        match self {
            Definition::Variable(_) => Vec::new(),
            Definition::Constant(definition) => definition.constraints.clone(),
            Definition::Type(definition) => definition.constraints.clone(),
            Definition::Trait(definition) => definition.constraints.clone(),
            Definition::Instance(definition) => definition.constraints.clone(),
            Definition::TypeParameter(_) => Vec::new(),
        }
    }
}

impl Visitor<'_> {
    fn push_scope(&mut self, _definition: NodeId) {
        self.scopes.push(Scope {
            definitions: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_name<'a, T: 'a>(
        &'a mut self,
        name: &str,
        node: NodeId,
        mut filter: impl FnMut(&'a Definition) -> Option<(T, Rule)>,
    ) -> Option<(T, Rule)> {
        let ((result, rule), definition) = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.definitions.get(name))
            .flatten()
            .find_map(|definition| Some((filter(definition)?, definition)))?;

        self.relations.add_edge(definition.source(), node, rule);

        Some((result, rule))
    }

    fn peek_name<'a, T: 'a>(
        &'a mut self,
        name: &str,
        filter: impl FnMut(&'a Definition) -> Option<T>,
    ) -> Option<T> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.definitions.get(name))
            .flatten()
            .find_map(filter)
    }

    fn define_name(&mut self, name: &str, definition: Definition) {
        self.scopes
            .last_mut()
            .unwrap()
            .definitions
            .entry(name.to_string())
            .or_default()
            .push(definition);
    }

    fn define_instance(&mut self, definition: InstanceDefinition) {
        self.instances
            .entry(definition.tr)
            .or_default()
            .push(definition);
    }

    fn with_implicit_type_parameters<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        assert!(!self.implicit_type_parameters);

        self.implicit_type_parameters = true;
        let result = f(self);
        self.implicit_type_parameters = false;

        result
    }
}
