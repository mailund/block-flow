use block_traits::type_erasure::Block;
pub use execution_context::ExecutionContext;
use intents::SlotIntent;
use weave_traits::TopoOrdered;

pub type TopoOrderedBlocks = TopoOrdered<Block>;

pub struct ExecutionPlan {
    blocks: TopoOrderedBlocks,
}

impl ExecutionPlan {
    pub fn new(blocks: TopoOrderedBlocks) -> Self {
        Self { blocks }
    }

    pub fn blocks(&self) -> &TopoOrderedBlocks {
        &self.blocks
    }

    pub fn execute(&self, context: &ExecutionContext) -> Vec<SlotIntent> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        self.blocks
            .iter()
            .flat_map(|block| block.execute(context))
            .collect()
    }
}
