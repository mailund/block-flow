use block_traits::{ExecutionContext, TopoOrderedBlocks};
use intents::Intent;

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

    pub fn execute(&self, context: &ExecutionContext) -> Vec<Intent> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        self.blocks
            .iter()
            .flat_map(|block| block.execute(context))
            .collect()
    }
}
