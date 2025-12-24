use block_traits::{Block, BlockTrait};
use execution_context::ExecutionContext;
use intents::SlotIntent;
use weave::TopoOrdered;

/// A mock implementation of an execution plan.
///
/// The plan can execute by executing its constituent blocks in topological order,
/// but it does not have any support for configuring it yet. That will likely require
/// a trait and is left for future work.
pub struct ExecutionPlan {
    block_id: u32,
    blocks: TopoOrdered<Block>,
}

impl ExecutionPlan {
    pub fn new(block_id: u32, blocks: TopoOrdered<Block>) -> Self {
        Self { block_id, blocks }
    }

    pub fn blocks(&self) -> &TopoOrdered<Block> {
        &self.blocks
    }
}

/// Implementing the BlockTrait so execution plans can be used as composite blocks.
impl BlockTrait for ExecutionPlan {
    fn block_id(&self) -> u32 {
        self.block_id
    }

    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        let mut deps = Vec::new();
        for block in self.blocks.iter() {
            deps.extend(block.contract_deps());
        }
        deps
    }

    // Collects and returns all SlotIntents produced by executing the blocks in the plan.
    // If any block fails to execute (returns None), the entire execution returns None.
    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
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
