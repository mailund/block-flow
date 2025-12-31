use super::execution_context::ExecutionContextTrait;
use super::ContractDeps;
use super::Effect;
use super::Intent;

/// Trait for consuming intents produced during block execution.
pub trait IntentConsumerTrait {
    /// Consume an intent. If the consumption fails, return a FailureStatus.
    fn consume(&mut self, intent: &Intent) -> Result<(), execute_status::FailureStatus>;
}

/// Trait for consuming effects produced during block execution.
pub trait EffectConsumerTrait {
    /// Consume an effect.
    fn schedule_effect(&mut self, effect: Effect) -> Result<(), execute_status::FailureStatus>;
    fn schedule_suspended_effect(&mut self) -> Result<(), execute_status::FailureStatus> {
        self.schedule_effect(Effect::Suspend)
    }
    fn schedule_terminate_effect(&mut self) -> Result<(), execute_status::FailureStatus> {
        self.schedule_effect(Effect::Terminate)
    }
    fn schedule_alarm_clock_effect(
        &mut self,
        new_time: u64,
    ) -> Result<(), execute_status::FailureStatus> {
        self.schedule_effect(Effect::Timer(new_time))
    }
}

/// An IntentConsumerTrait implementation that wraps a closure which may fail.
/// The new-type is needed to avoid conflicting implementations for all closures
/// (i.e., the blanket implementation below for FnMut(&intent)).
pub struct FaillibleIntentConsumer<F>(F);
impl<F> IntentConsumerTrait for FaillibleIntentConsumer<F>
where
    F: FnMut(&Intent) -> Result<(), execute_status::FailureStatus>,
{
    fn consume(&mut self, intent: &Intent) -> Result<(), execute_status::FailureStatus> {
        (self.0)(intent)
    }
}

/// Implement IntentConsumerTrait for any closure that matches the signature.
impl<F> IntentConsumerTrait for F
where
    F: FnMut(&Intent),
{
    fn consume(&mut self, intent: &Intent) -> Result<(), execute_status::FailureStatus> {
        self(intent);
        Ok(())
    }
}

/// Implement EffectConsumerTrait for any closure that matches the signature.
pub struct FaillibleEffectConsumer<F>(F);
impl<F> EffectConsumerTrait for FaillibleEffectConsumer<F>
where
    F: FnMut(Effect) -> Result<(), execute_status::FailureStatus>,
{
    fn schedule_effect(&mut self, effect: Effect) -> Result<(), execute_status::FailureStatus> {
        (self.0)(effect)
    }
}
impl<F> EffectConsumerTrait for F
where
    F: FnMut(Effect),
{
    fn schedule_effect(&mut self, effect: Effect) -> Result<(), execute_status::FailureStatus> {
        self(effect);
        Ok(())
    }
}

/// Module defining execution status types.
///
/// Currently, this is at the sophistication level of Success/Failure,
/// and could also be represented as Result<(), ()> or Option<()>,
/// but having explicit types allows for future expansion.
pub mod execute_status {
    #[derive(Debug, Clone)]
    pub enum SuccessStatus {
        Success,
    }
    pub use SuccessStatus::Success;

    #[derive(Debug, Clone)]
    pub enum FailureStatus {
        Failure,
    }
    pub use FailureStatus::Failure;
    pub type ExecuteResult = std::result::Result<SuccessStatus, FailureStatus>;
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
    fn num_intents(&self) -> usize;
    /// Execute the block in the given execution context, producing intents consumed by the intent consumer.
    fn execute(
        &self,
        context: &ExeContext,
        intent_consumer: &mut IntentConsumer,
        effect_consumer: &mut EffectConsumer,
    ) -> execute_status::ExecuteResult;
}
