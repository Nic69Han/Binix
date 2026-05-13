use taffy::prelude::*;
use binix_core::Result;

pub struct LayoutEngine { taffy: TaffyTree<()> }

impl LayoutEngine {
    pub fn new() -> Self { Self { taffy: TaffyTree::new() } }
    pub fn compute(&mut self) -> Result<()> {
        self.taffy.compute_layout(taffy::NodeId::root(), Size::MAX_CONTENT).unwrap();
        Ok(())
    }
}
