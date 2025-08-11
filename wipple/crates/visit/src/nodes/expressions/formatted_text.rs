use crate::visitor::{Visit, Visitor};
use wipple_db::NodeId;
use wipple_syntax::{FormattedTextExpression, Range};
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
