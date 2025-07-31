use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{AnnotatePattern, Range};
use wipple_compiler_trace::{NodeId, Rule};
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for AnnotatePattern {
    fn rule(&self) -> Rule {
        "annotate pattern".into()
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let pattern = visitor.child(self.left.as_ref(), id, "annotated pattern");
        let ty = visitor.child(&self.right, pattern, "type in annotated pattern");

        visitor.constraint(Constraint::Ty(pattern, Ty::Of(ty)));
        visitor.constraint(Constraint::Ty(id, Ty::Of(ty)));
    }

    fn is_typed(&self) -> bool {
        true
    }
}
