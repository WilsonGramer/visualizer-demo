use crate::visitor::{Visit, Visitor};
use visualizer::db::NodeId;
use visualizer::typecheck::{Constraint, Ty};
use wipple_syntax::{AnnotatePattern, Range};

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
