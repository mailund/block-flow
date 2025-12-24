use super::execution_context::ExecutionContext;
use super::intents::SlotIntent;

pub trait BlockTrait {
    fn block_id(&self) -> u32;
    fn contract_deps(&self) -> Vec<::trade_types::Contract>;
    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>>;
}

/// Type-erased block for execution in a weaved execution plan.
pub struct Block {
    block: Box<dyn BlockTrait>,
}

impl Block {
    pub fn new(block: Box<dyn BlockTrait>) -> Self {
        Self { block }
    }
}

impl BlockTrait for Block {
    fn block_id(&self) -> u32 {
        self.block.block_id()
    }

    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.block.contract_deps()
    }

    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        self.block.execute(context)
    }
}
