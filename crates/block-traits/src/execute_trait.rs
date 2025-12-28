use super::execution_context::ExecutionContextTrait;
use super::ContractDeps;
use super::Intent;

/// Trait necessary to execute a type-erased block.
/// Captures anything that can be executed in a given execution context
/// and that will return slot intents upon execution.
pub trait ExecuteTrait<Context: ExecutionContextTrait>: ContractDeps {
    fn execute(&self, context: &Context) -> Option<Vec<Intent>>;
}
