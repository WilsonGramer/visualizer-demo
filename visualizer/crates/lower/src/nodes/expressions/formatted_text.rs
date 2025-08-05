use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{FormattedTextExpression, Range};
use wipple_visualizer_typecheck::NodeId;
// TODO

impl Visit for FormattedTextExpression {
    fn name(&self) -> &'static str {
        "formattedText"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
