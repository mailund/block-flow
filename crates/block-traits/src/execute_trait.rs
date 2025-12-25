use super::execution_context::ExecutionContextTrait;
use super::intents::SlotIntent;
use super::ContractDeps;

/// Trait necessary to execute a type-erased block.
/// Captures anything that can be executed in a given execution context
/// and that will return slot intents upon execution.
pub trait ExecuteTrait<Context: ExecutionContextTrait>: ContractDeps {
    fn execute(&self, context: &Context) -> Option<Vec<SlotIntent>>;
}
