use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{AnnotateExpression, Range};
use wipple_visualizer_typecheck::{
    Constraint, Ty,
    NodeId,
};

impl Visit for AnnotateExpression {
    fn name(&self) -> &'static str {
        "annotate"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        let value = visitor.child(self.left.as_ref(), id, "annotatedValue");
        let ty = visitor.child(&self.right, value, "typeInAnnotatedValue");

        visitor.constraint(Constraint::Ty(value, Ty::Of(ty)));
        visitor.constraint(Constraint::Ty(id, Ty::Of(ty)));
    }
}
