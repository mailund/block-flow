use crate::intents::SlotIntent;
use crate::{Block, BlockTrait, ExecutionContext};
use ::channels::weave::TopoOrdered;

/// A mock implementation of an execution plan.
///
/// The plan can execute by executing its constituent blocks in topological order,
/// but it does not have any support for configuring it yet. That will likely require
/// a trait and is left for future work.
pub struct ExecutionPlan {
    blocks: TopoOrdered<Block>,
}

impl ExecutionPlan {
    pub fn new(blocks: TopoOrdered<Block>) -> Self {
        Self { blocks }
    }

    pub fn blocks(&self) -> &TopoOrdered<Block> {
        &self.blocks
    }
}

/// Implementing the BlockTrait so execution plans can be used as composite blocks.
impl BlockTrait for TopoOrdered<Block> {
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        let mut deps = Vec::new();
        for block in self.iter() {
            deps.extend(block.contract_deps());
        }
        deps
    }

    // Collects and returns all SlotIntents produced by executing the blocks in the plan.
    // If any block fails to execute (returns None), the entire execution returns None.
    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        self.iter()
            // Get the optional output for each block
            .map(|block| block.execute(context))
            // Short-circuit if any block returned None
            .collect::<Option<Vec<Vec<SlotIntent>>>>()
            // Flatten the Vec<Vec<SlotIntent>> into Vec<SlotIntent>
            .map(|v| v.into_iter().flatten().collect())
    }
}

impl BlockTrait for ExecutionPlan {
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.blocks.contract_deps()
    }

    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        self.blocks.execute(context)
    }
}
