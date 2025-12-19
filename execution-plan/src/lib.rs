use block_traits::type_erasure::Block;
use execution_context::ExecutionContext;
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

    /// Collects and returns all SlotIntents produced by executing the blocks in the plan.
    /// If any block fails to execute (returns None), the entire execution returns None.
    pub fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        self.blocks
            .iter()
            // Get the optional output for each block
            .map(|block| block.execute(context))
            // Short-circuit if any block returned None
            .collect::<Option<Vec<Vec<SlotIntent>>>>()
            // Flatten the Vec<Vec<SlotIntent>> into Vec<SlotIntent>
            .map(|v| v.into_iter().flatten().collect())
    }
}
