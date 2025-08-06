use std::{collections::BTreeMap, fmt::Debug};
use visualizer_db::{Db, FactValue, NodeId};

#[derive(Debug, Clone)]
pub enum Constraint {
    Ty(NodeId, Ty),
    Instantiation(Instantiation),
    Bound(Bound),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Unknown,
    Of(NodeId),
    Parameter(NodeId),
    Named {
        name: NodeId,
        parameters: BTreeMap<NodeId, Ty>,
    },
    Function {
        inputs: Vec<Ty>,
        output: Box<Ty>,
    },
    Tuple {
        elements: Vec<Ty>,
    },
}

impl Ty {
    pub fn unit() -> Self {
        Ty::Tuple {
            elements: Vec::new(),
        }
    }

    pub fn traverse(&self, f: &mut impl FnMut(&Self)) {
        f(self);

        match self {
            Ty::Unknown | Ty::Of(_) | Ty::Parameter(_) => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters.values() {
                    parameter.traverse(f);
                }
            }
            Ty::Function { inputs, output } => {
                for input in inputs {
                    input.traverse(f);
                }

                output.traverse(f);
            }
            Ty::Tuple { elements } => {
                for element in elements {
                    element.traverse(f);
                }
            }
        }
    }

    pub fn traverse_mut(&mut self, f: &mut impl FnMut(&mut Self)) {
        f(self);

        match self {
            Ty::Unknown | Ty::Of(_) | Ty::Parameter(_) => {}
            Ty::Named { parameters, .. } => {
                for parameter in parameters.values_mut() {
                    parameter.traverse_mut(f);
                }
            }
            Ty::Function { inputs, output } => {
                for input in inputs {
                    input.traverse_mut(f);
                }

                output.traverse_mut(f);
            }
            Ty::Tuple { elements } => {
                for element in elements {
                    element.traverse_mut(f);
                }
            }
        }
    }

    pub fn is_incomplete(&self) -> bool {
        let mut incomplete = false;
        self.traverse(&mut |ty| {
            if matches!(ty, Ty::Of(_)) {
                incomplete = true;
            }
        });

        incomplete
    }
}

impl FactValue for Ty {
    fn display(&self, db: &Db) -> Option<String> {
        Some(match self {
            Ty::Unknown | Ty::Of(_) => String::from("_"),
            Ty::Parameter(node) => db.get::<String>(*node, "source").unwrap().clone(),
            Ty::Named { name, parameters } => format!(
                "{}{}",
                db.get::<String>(*name, "source").unwrap(),
                parameters
                    .values()
                    .map(|parameter| format!(" {}", parameter.display(db).unwrap()))
                    .collect::<String>()
            ),
            Ty::Function { inputs, output } => format!(
                "{}-> {}",
                inputs
                    .iter()
                    .map(|input| format!("{} ", input.display(db).unwrap()))
                    .collect::<String>(),
                output.display(db).unwrap()
            ),
            Ty::Tuple { elements } => format!(
                "({})",
                elements
                    .iter()
                    .map(|element| element.display(db).unwrap())
                    .collect::<Vec<_>>()
                    .join(" ; ")
            ),
        })
    }

    fn is_code(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Substitutions(pub BTreeMap<NodeId, Ty>);

impl Substitutions {
    pub fn replace_all() -> Self {
        Substitutions(BTreeMap::new())
    }
}

impl From<BTreeMap<NodeId, NodeId>> for Substitutions {
    fn from(value: BTreeMap<NodeId, NodeId>) -> Self {
        Substitutions(
            value
                .into_iter()
                .map(|(parameter, node)| (parameter, Ty::Of(node)))
                .collect(),
        )
    }
}

impl FactValue for Substitutions {
    fn display(&self, _db: &Db) -> Option<String> {
        Some(String::from("Substitutions(..)"))
    }
}

#[derive(Debug, Clone)]
pub struct Instantiation {
    pub substitutions: Substitutions,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Bound {
    pub node: NodeId,
    pub tr: NodeId,
    pub substitutions: Substitutions,
}
