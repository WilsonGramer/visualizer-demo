use crate::visitor::{Visit, Visitor};
use wipple_compiler_syntax::{AnnotatePattern, Range};
use wipple_compiler_trace::NodeId;
use wipple_compiler_typecheck::constraints::{Constraint, Ty};

impl Visit for AnnotatePattern {
    fn name(&self) -> &'static str {
        "annotatePattern"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let pattern = visitor.child(self.left.as_ref(), id, "annotatedPattern");
        let ty = visitor.child(&self.right, pattern, "typeInAnnotatedPattern");

        visitor.constraint(Constraint::Ty(pattern, Ty::Of(ty)));
        visitor.constraint(Constraint::Ty(id, Ty::Of(ty)));
    }
}
