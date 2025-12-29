use super::execution_context::ExecutionContextTrait;
use super::ContractDeps;
use super::Effect;
use super::Intent;

/// Trait for consuming intents produced during block execution.
pub trait IntentConsumerTrait {
    /// Consume an intent.
    fn consume(&mut self, intent: &Intent);
}

/// Trait for consuming effects produced during block execution.
pub trait EffectConsumerTrait {
    /// Consume an effect.
    fn schedule_effect(&mut self, effect: Effect);
    fn schedule_suspended_effect(&mut self) {
        self.schedule_effect(Effect::Suspend);
    }
    fn schedule_terminate_effect(&mut self) {
        self.schedule_effect(Effect::Terminate);
    }
    fn schedule_alarm_clock_effect(&mut self, new_time: u64) {
        self.schedule_effect(Effect::Timer(new_time));
    }
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

/// Implement EffectConsumerTrait for any closure that matches the signature.
impl<F> EffectConsumerTrait for F
where
    F: FnMut(Effect),
{
    fn schedule_effect(&mut self, effect: Effect) {
        self(effect);
    }
}

/// Trait necessary to execute a type-erased block.
/// Captures anything that can be executed in a given execution context
/// and that will return slot intents upon execution.
pub trait ExecuteTrait<ExeContext, IntentConsumer, EffectConsumer>: ContractDeps
where
    ExeContext: ExecutionContextTrait,
    IntentConsumer: IntentConsumerTrait + ?Sized,
    EffectConsumer: EffectConsumerTrait + ?Sized,
{
    /// Number of intents produced by the execution.
    /// This should be a constant once the execution trait is instantiated, but since we can build
    /// algorithms dynamically it is not possible to enforce this at compile time.
    fn no_intents(&self) -> usize;
    /// Execute the block in the given execution context, producing intents consumed by the intent consumer.
    fn execute(
        &self,
        context: &ExeContext,
        intent_consumer: &mut IntentConsumer,
        effect_consumer: &mut EffectConsumer,
    ) -> Option<()>;
}
