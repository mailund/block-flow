use super::execution_context::ExecutionContextTrait;
use super::ContractDeps;
use super::Intent;

pub trait IntentConsumerTrait {
    fn consume(&mut self, intent: &Intent);
}

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
    fn execute(&self, context: &ExeContext, consumer: &mut IntentConsumer) -> Option<()>;
}
