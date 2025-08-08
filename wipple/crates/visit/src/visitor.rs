use crate::definitions::{Definition, InstanceDefinition};
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};
use visualizer::Constraint;
use wipple_db::{Db, Fact, FactValue, NodeId, Span};
use wipple_syntax::{self as syntax, Range};

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
    fn name(&self) -> &'static str;

    fn range(&self) -> Range;

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>);

    fn hide(&self) -> bool {
        false
    }
}

impl Visit for (Range, &'static str) {
    fn name(&self) -> &'static str {
        self.1
    }

    fn range(&self) -> Range {
        self.0
    }

    fn visit(&self, _id: NodeId, _visitor: &mut Visitor<'_>) {}
}

pub struct ProgramInfo {
    pub definitions: BTreeMap<NodeId, Definition>,
    pub instances: BTreeMap<NodeId, Vec<NodeId>>,
    pub definition_constraints: Vec<Constraint<Db>>,
    pub top_level_constraints: Vec<Constraint<Db>>,
}

pub struct Visitor<'a> {
    db: &'a mut Db,
    get_span_source: Box<dyn Fn(Range) -> (Span, String) + 'a>,
    scopes: Vec<Scope>,
    instances: BTreeMap<NodeId, Vec<InstanceDefinition>>,
    definition_constraints: Vec<Constraint<Db>>,
    program_constraints: Vec<Constraint<Db>>,
    current_definition: Option<VisitorCurrentDefinition>,
}

impl<'a> Visitor<'a> {
    pub fn new(db: &'a mut Db, get_span_source: impl Fn(Range) -> (Span, String) + 'a) -> Self {
        Visitor {
            db,
            get_span_source: Box::new(get_span_source),
            scopes: vec![Scope::default()],
            instances: Default::default(),
            definition_constraints: Default::default(),
            program_constraints: Default::default(),
            current_definition: None,
        }
    }

    pub fn finish(mut self) -> ProgramInfo {
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

        for node in definitions.keys().copied() {
            self.db.fact(node, Fact::new("untyped", ()));
        }

        ProgramInfo {
            definitions,
            instances: instance_ids,
            definition_constraints: self.definition_constraints,
            top_level_constraints: self.program_constraints,
        }
    }
}

impl<'a> Visitor<'a> {
    pub fn node(&mut self, range: Range, name: &'static str) -> NodeId {
        let node = self.db.node();

        self.fact(node, name, ());

        let (span, source) = (self.get_span_source)(range);
        self.fact(node, "span", span);
        self.fact(node, "source", source);

        node
    }

    pub fn fact(&mut self, node: NodeId, name: impl AsRef<str>, value: impl FactValue) {
        self.db.fact(node, Fact::new(name, value));
    }

    pub fn hide(&mut self, node: NodeId) {
        self.db.fact(node, Fact::hidden());
    }

    pub fn child(&mut self, node: &impl Visit, parent: NodeId, relation: &'static str) -> NodeId {
        let id = self.node(node.range(), node.name());

        self.relation(id, parent, relation);

        if node.hide() {
            self.hide(id);
        }

        node.visit(id, self);

        id
    }

    pub fn relation(&mut self, child: NodeId, parent: NodeId, relation: &'static str) {
        self.fact(child, relation, parent);
    }

    pub fn constraint(&mut self, constraint: Constraint<Db>) {
        if self.current_definition.is_some() {
            self.definition_constraints.push(constraint);
        } else {
            self.program_constraints.push(constraint);
        }
    }

    pub fn constraints(&mut self, constraints: impl IntoIterator<Item = Constraint<Db>>) {
        for constraint in constraints {
            self.constraint(constraint);
        }
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

    pub fn resolve_name<T>(
        &mut self,
        name: &str,
        node: NodeId,
        mut filter: impl FnMut(&Definition) -> Option<(T, &'static str)>,
    ) -> Option<T> {
        let ((result, relation), definition) = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.definitions.get(name))
            .flatten()
            .find_map(|definition| Some((filter(definition)?, definition.source())))?;

        self.relation(node, definition, relation);

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

pub type LazyConstraint = Rc<dyn Fn(NodeId) -> Constraint<Db>>;

impl VisitorCurrentDefinition {
    pub fn constraint(&mut self, constraint: Constraint<Db>) {
        self.lazy_constraint(move |_| constraint.clone());
    }

    pub fn lazy_constraint(&mut self, constraint: impl Fn(NodeId) -> Constraint<Db> + 'static) {
        self.constraints.push(Rc::new(constraint));
    }

    pub fn take_constraints(&mut self) -> Vec<LazyConstraint> {
        std::mem::take(&mut self.constraints)
    }
}
