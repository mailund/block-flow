use super::execution_context::ExecutionContextTrait;
use super::ContractDeps;
use super::Intent;

/// Trait for consuming intents produced during block execution.
pub trait IntentConsumerTrait {
    /// Consume an intent.
    fn consume(&mut self, intent: &Intent);
}

/// Implement IntentConsumerTrait for any closure that matches the signature.
impl<F> IntentConsumerTrait for F
where
    F: FnMut(&Intent),
{
    fn consume(&mut self, intent: &Intent) {
        self(intent);
    }
}

/// Trait necessary to execute a type-erased block.
/// Captures anything that can be executed in a given execution context
/// and that will return slot intents upon execution.
pub trait ExecuteTrait<ExeContext, IntentConsumer>: ContractDeps
where
    ExeContext: ExecutionContextTrait,
    IntentConsumer: IntentConsumerTrait,
{
    /// Number of intents produced by the execution.
    /// This should be a constant once the execution trait is instantiated, but since we can build
    /// algorithms dynamically it is not possible to enforce this at compile time.
    fn no_intents(&self) -> usize;
    /// Execute the block in the given execution context, producing intents consumed by the intent consumer.
    fn execute(&self, context: &ExeContext, consumer: &mut IntentConsumer) -> Option<()>;
}
