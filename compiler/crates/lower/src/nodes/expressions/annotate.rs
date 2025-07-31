use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{AnnotateExpression, Range};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for AnnotateExpression {
    fn rule(&self) -> Rule {
        "annotate".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let value = visitor.child(self.left.as_ref(), id, "annotated value");
        let ty = visitor.child(&self.right, value, "type in annotated value");

        visitor.constraint(Constraint::Ty(value, Ty::Of(ty)));
        visitor.constraint(Constraint::Ty(value, Ty::Of(id)));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
