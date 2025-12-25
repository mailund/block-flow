use super::execution_context::ExecutionContext;
use super::intents::SlotIntent;

/// Trait necessary to execute a type-erased block.
pub trait BlockTrait {
    fn contract_deps(&self) -> Vec<::trade_types::Contract>;
    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>>;
}
