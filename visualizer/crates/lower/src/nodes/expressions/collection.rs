use crate::visitor::{Visit, Visitor};
use wipple_visualizer_syntax::{CollectionExpression, Range};
use wipple_visualizer_typecheck::NodeId;
// TODO

impl Visit for CollectionExpression {
    fn name(&self) -> &'static str {
        "collection"
    }

    fn range(&self) -> Range {
        self.range
    }

    fn visit(&self, id: NodeId, visitor: &mut Visitor<'_>) {
        todo!()
    }

}
