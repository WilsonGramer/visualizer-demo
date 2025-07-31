use crate::definitions::{Definition, InstanceDefinition};
use petgraph::prelude::DiGraphMap;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
};
use wipple_compiler_syntax::{self as syntax, Range};
use wipple_compiler_trace::{NodeId, Rule, Span};
use wipple_compiler_typecheck::constraints::Constraint;

#[enum_delegate::register]
#[enum_delegate::implement_for(syntax::Constraint, enum Constraint {
    Bound(BoundConstraint),
    Infer(InferConstraint),
    Default(DefaultConstraint),
})]
#[enum_delegate::implement_for(syntax::Expression, enum Expression {
    Function(FunctionExpression),
    Tuple(TupleExpression),
    Collection(CollectionExpression),
    Is(IsExpression),
    As(AsExpression),
    Annotate(AnnotateExpression),
    Binary(BinaryExpression),
    FormattedText(FormattedTextExpression),
    Call(CallExpression),
    Do(DoExpression),
    When(WhenExpression),
    Intrinsic(IntrinsicExpression),
    Placeholder(PlaceholderExpression),
    Variable(VariableExpression),
    Trait(TraitExpression),
    Number(NumberExpression),
    Text(TextExpression),
    Structure(StructureExpression),
    Block(BlockExpression),
    Unit(UnitExpression),
})]
#[enum_delegate::implement_for(syntax::Pattern, enum Pattern {
    Unit(UnitPattern),
    Wildcard(WildcardPattern),
    Variable(VariablePattern),
    Number(NumberPattern),
    Text(TextPattern),
    Destructure(DestructurePattern),
    Set(SetPattern),
    Variant(VariantPattern),
    Or(OrPattern),
    Tuple(TuplePattern),
    Annotate(AnnotatePattern),
})]
#[enum_delegate::implement_for(syntax::Statement, enum Statement {
    ConstantDefinition(ConstantDefinitionStatement),
    TypeDefinition(TypeDefinitionStatement),
    TraitDefinition(TraitDefinitionStatement),
    InstanceDefinition(InstanceDefinitionStatement),
    Assignment(AssignmentStatement),
    Expression(ExpressionStatement),
    Empty(EmptyStatement),
})]
#[enum_delegate::implement_for(syntax::Type, enum Type {
    Placeholder(PlaceholderType),
    Unit(UnitType),
    Named(NamedType),
    Parameterized(ParameterizedType),
    Block(BlockType),
    Function(FunctionType),
    Parameter(ParameterType),
    Tuple(TupleType),
})]
pub trait Visit {
    fn rule(&self) -> Rule;

    fn range(&self) -> Range;

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>);

    fn is_typed(&self) -> bool {
        false
    }
}

impl Visit for (Range, Rule) {
    fn rule(&self) -> Rule {
        self.1
    }

    fn range(&self) -> Range {
        self.0
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}

pub struct Result {
    pub next_id: NodeId,
    pub typed_nodes: BTreeSet<NodeId>,
    pub spans: BTreeMap<NodeId, Span>,
    pub rules: BTreeMap<NodeId, HashSet<Rule>>,
    pub relations: DiGraphMap<NodeId, Rule>,
    pub definitions: BTreeMap<NodeId, Definition>,
    pub instances: BTreeMap<NodeId, Vec<NodeId>>,
    pub constraints: Vec<Constraint>,
}

pub struct Visitor<'a> {
    make_span: Box<dyn Fn(Range) -> Span + 'a>,
    next_id: NodeId,
    typed_nodes: BTreeSet<NodeId>,
    spans: BTreeMap<NodeId, Span>,
    rules: BTreeMap<NodeId, HashSet<Rule>>,
    relations: DiGraphMap<NodeId, Rule>,
    scopes: Vec<Scope>,
    instances: BTreeMap<NodeId, Vec<InstanceDefinition>>,
    constraints: Vec<Constraint>,
    current_definition: Option<VisitorCurrentDefinition>,
}

impl<'a> Visitor<'a> {
    pub fn new(make_span: impl Fn(Range) -> Span + 'a) -> Self {
        Visitor {
            next_id: NodeId(0),
            make_span: Box::new(make_span),
            typed_nodes: Default::default(),
            spans: Default::default(),
            rules: Default::default(),
            relations: Default::default(),
            scopes: vec![Scope::default()],
            instances: Default::default(),
            constraints: Default::default(),
            current_definition: None,
        }
    }

    pub fn finish(mut self) -> Result {
        let mut definitions = self
            .scopes
            .pop()
            .unwrap()
            .definitions
            .into_values()
            .flatten()
            .map(|definition| (definition.source(), definition))
            .collect::<BTreeMap<_, _>>();

        let mut instance_ids = BTreeMap::<_, Vec<_>>::new();
        for (tr, instances) in self.instances {
            for instance in instances {
                definitions.insert(instance.node, Definition::Instance(instance.clone()));
                instance_ids.entry(tr).or_default().push(instance.node);
            }
        }

        Result {
            next_id: self.next_id,
            typed_nodes: self.typed_nodes,
            spans: self.spans,
            relations: self.relations,
            definitions,
            instances: instance_ids,
            constraints: self.constraints,
            rules: self.rules,
        }
    }
}

impl<'a> Visitor<'a> {
    pub fn node(&mut self, range: Range, rule: impl Into<Rule>) -> NodeId {
        let id = self.next_id;
        self.next_id.0 += 1;

        self.spans.insert(id, (self.make_span)(range));
        self.rule(id, rule);

        id
    }

    pub fn rule(&mut self, node: NodeId, rule: impl Into<Rule>) {
        self.rules.entry(node).or_default().insert(rule.into());
    }

    pub fn child(
        &mut self,
        node: &impl Visit,
        parent: NodeId,
        relation: impl Into<Rule>,
    ) -> NodeId {
        let id = self.node(node.range(), node.rule());

        self.relations.add_edge(parent, id, relation.into());

        if node.is_typed() {
            self.typed_nodes.insert(id);
        }

        node.visit(id, self);

        id
    }

    pub fn constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn constraints(&mut self, constraints: impl IntoIterator<Item = Constraint>) {
        self.constraints.extend(constraints);
    }
}

#[derive(Clone, Default)]
struct Scope {
    definitions: HashMap<String, Vec<Definition>>,
}

impl Visitor<'_> {
    pub fn push_scope(&mut self, _definition: NodeId) {
        self.scopes.push(Scope {
            definitions: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_name<'a, T: 'a, R: Into<Rule>>(
        &'a mut self,
        name: &str,
        node: NodeId,
        mut filter: impl FnMut(&'a Definition) -> Option<(T, R)>,
    ) -> Option<T> {
        let ((result, rule), definition) = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.definitions.get(name))
            .flatten()
            .find_map(|definition| Some((filter(definition)?, definition)))?;

        self.relations
            .add_edge(definition.source(), node, rule.into());

        Some(result)
    }

    pub fn peek_name<'a, T: 'a>(
        &'a mut self,
        name: &str,
        filter: impl FnMut(&'a mut Definition) -> Option<T>,
    ) -> Option<T> {
        self.scopes
            .iter_mut()
            .rev()
            .filter_map(|scope| scope.definitions.get_mut(name))
            .flatten()
            .find_map(filter)
    }

    pub fn define_name(&mut self, name: &str, definition: Definition) {
        self.scopes
            .last_mut()
            .unwrap()
            .definitions
            .entry(name.to_string())
            .or_default()
            .push(definition);
    }

    pub fn define_instance(&mut self, definition: InstanceDefinition) {
        self.instances
            .entry(definition.tr)
            .or_default()
            .push(definition);
    }

    pub fn with_definition<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let existing = self.current_definition.replace(Default::default());
        let result = f(self);
        self.current_definition = existing;

        result
    }

    pub fn try_current_definition(&mut self) -> Option<&mut VisitorCurrentDefinition> {
        self.current_definition.as_mut()
    }

    pub fn current_definition(&mut self) -> &mut VisitorCurrentDefinition {
        self.try_current_definition()
            .expect("no current definition")
    }
}

#[derive(Default)]
pub struct VisitorCurrentDefinition {
    constraints: Vec<LazyConstraint>,
    pub implicit_type_parameters: bool,
}

pub type LazyConstraint = Rc<dyn Fn(NodeId) -> Constraint>;

impl VisitorCurrentDefinition {
    pub fn constraint(&mut self, constraint: Constraint) {
        self.lazy_constraint(move |_| constraint.clone());
    }

    pub fn lazy_constraint(&mut self, constraint: impl Fn(NodeId) -> Constraint + 'static) {
        self.constraints.push(Rc::new(constraint));
    }

    pub fn take_constraints(&mut self) -> Vec<LazyConstraint> {
        std::mem::take(&mut self.constraints)
    }
}
