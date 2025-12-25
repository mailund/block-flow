use super::execution_context::ExecutionContextTrait;
use super::intents::SlotIntent;
use super::ContractDeps;

/// Trait necessary to execute a type-erased block.
pub trait BlockTrait<Context: ExecutionContextTrait>: ContractDeps {
    fn execute(&self, context: &Context) -> Option<Vec<SlotIntent>>;
}
