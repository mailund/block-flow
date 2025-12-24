use super::execution_context::ExecutionContext;
use super::intents::SlotIntent;

/// Trait necessary to execute a type-erased block.
pub trait BlockExecuteTrait {
    fn contract_deps(&self) -> Vec<::trade_types::Contract>;
    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>>;
}

/// Additional traits for blocks we can identify by block ID.
pub trait BlockTrait: BlockExecuteTrait {
    fn block_id(&self) -> u32;
}

/// Type-erased block with execution capabilities.
pub struct ExecuteBlock(Box<dyn BlockExecuteTrait>);

impl ExecuteBlock {
    pub fn new(block: Box<dyn BlockTrait>) -> Self {
        Self(block)
    }
}

impl BlockExecuteTrait for ExecuteBlock {
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.0.contract_deps()
    }

    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        self.0.execute(context)
    }
}

/// Type-erased block for execution in a weaved execution plan.
pub struct Block(Box<dyn BlockTrait>);

impl Block {
    pub fn new(block: Box<dyn BlockTrait>) -> Self {
        Self(block)
    }
}

impl BlockExecuteTrait for Block {
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.0.contract_deps()
    }

    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        self.0.execute(context)
    }
}

impl BlockTrait for Block {
    fn block_id(&self) -> u32 {
        self.0.block_id()
    }
}
