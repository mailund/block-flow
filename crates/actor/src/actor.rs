use super::*;
use block_traits::{
    execute_status, Effect, EffectConsumerTrait, ExecuteTrait, Intent, IntentConsumerTrait,
};
use trade_types::Contract;

/// Marker trait for “things we can run as an actor algorithm”.
///
/// This is intentionally just a *type alias for a trait bound*:
/// any type that implements `ExecuteTrait<ActorExecutionContext, …>` with the module’s
/// intent/effect consumer types automatically becomes an `ActorAlgo`.
///
/// Why this exists:
/// - keeps `Actor<Algo>` bounds readable
/// - locks the actor runtime to *these* consumer implementations
/// - avoids leaking consumer details into unrelated crates’ bounds (only `Actor` cares)
///
/// The `for<'a>` bound is required because both consumers borrow internal buffers (`orders` / `effects`)
/// inside the actor for the duration of a single `execute` call.
pub trait ActorAlgo:
    for<'a> ExecuteTrait<ActorExecutionContext, ReconcileIntentConsumer<'a>, EffectConsumer<'a>>
{
}

// Implementing the ActorAlgo for all matching types.
//
// This makes it such that an actor can take any implementation of the ExecuteTrait
// with the correct context and consumers as its algorithm and use it as an ActorAlgo.
// In effect it makes the ActorAlgo trait a type alias (while trait aliases are not yet
// supported in stable Rust).
impl<T> ActorAlgo for T where
    T: for<'a> ExecuteTrait<ActorExecutionContext, ReconcileIntentConsumer<'a>, EffectConsumer<'a>>
{
}

/// Owns the “order state” buffer used by reconciliation.
///
/// The reconciliator is created once in `Actor::new()` sized to `algo.num_intents()`.
/// Each tick, we create a fresh `ReconcileIntentConsumer<'_>` that borrows the buffer
/// and starts at `idx = 0` (so an algorithm can be executed repeatedly without having
/// to remember to reset indices).
struct Reconciliator {
    orders: Vec<Order>,
}
impl Reconciliator {
    pub fn new(size: usize) -> Self {
        Self {
            orders: vec![Order::default(); size],
        }
    }

    /// Borrow the internal order buffer for the duration of a single `execute` call.
    ///
    /// Each call creates a new consumer with `idx = 0`.
    pub fn intent_consumer(&mut self) -> ReconcileIntentConsumer<'_> {
        ReconcileIntentConsumer::new(&mut self.orders)
    }
}

/// Intent consumer used during algorithm execution.
///
/// The algorithm produces `Intent`s by calling `IntentConsumerTrait::consume`.
/// This consumer translates intents into an updated order buffer.
///
/// Important invariants:
/// - `idx` advances once per consumed intent
/// - the buffer length must be at least the maximum number of intents the algorithm can emit
///   (the actor enforces this by sizing from `algo.num_intents()`).
pub struct ReconcileIntentConsumer<'a> {
    orders: &'a mut [Order],
    idx: usize,
}
impl<'a> ReconcileIntentConsumer<'a> {
    /// Create a new consumer over an existing order buffer.
    ///
    /// The buffer is borrowed mutably and updated in-place. The consumer always starts at `idx = 0`.
    pub fn new(orders: &'a mut [Order]) -> Self {
        Self { orders, idx: 0 }
    }

    /// Convert an intent into the next order state given the previous order state.
    ///
    /// This is deliberately “mock-simple” right now:
    /// - `NoIntent` keeps the previous order
    /// - `Place` overwrites with a new order request
    fn process_intent(&self, _prev_order: &Order, intent: &Intent) -> Order {
        match intent {
            Intent::NoIntent => Order::NoOrder,
            Intent::Place {
                contract,
                side,
                price,
                quantity,
            } => Order::New {
                contract: contract.clone(),
                side: side.clone(),
                price: price.clone(),
                quantity: quantity.clone(),
            },
        }
    }

    /// Consume one intent and update one slot of the order buffer.
    ///
    /// Panics if the algorithm emits more intents than the buffer length.
    fn consume(&mut self, intent: &Intent) -> Result<(), execute_status::FailureStatus> {
        self.orders[self.idx] = self.process_intent(&self.orders[self.idx], intent);
        self.idx += 1;
        Ok(())
    }
}

impl<'a> IntentConsumerTrait for ReconcileIntentConsumer<'a> {
    fn consume(&mut self, intent: &Intent) -> Result<(), execute_status::FailureStatus> {
        ReconcileIntentConsumer::consume(self, intent)
    }
}

/// Owns the effect buffer used by the effect consumer.
///
/// Effects are collected during execution and can be handled afterwards.
/// The effect buffer is cleared at the start of each tick (via `EffectConsumer::new()`).
struct EffectHandler {
    effects: Vec<Effect>,
}
impl EffectHandler {
    /// Create a new effect handler with an empty effect buffer.
    fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Borrow the internal effect buffer for the duration of a single `execute` call.
    ///
    /// The buffer is cleared at the start of each call.
    fn effect_consumer(&mut self) -> EffectConsumer<'_> {
        EffectConsumer::new(&mut self.effects)
    }

    /// Handle all effects collected during the last execution.
    fn handle_effects(&self) -> execute_status::ExecuteResult {
        for effect in self.effects.iter() {
            println!("Actor {} handling effect: {:?}", -122, effect);
        }
        Ok(execute_status::Success)
    }
}

/// Effect consumer used during algorithm execution.
///
/// The algorithm schedules effects (side-effects) via `EffectConsumerTrait`.
/// Effects are collected in a buffer owned by the actor. The buffer is cleared
/// at the start of each tick to ensure effects from previous ticks do not leak.
pub struct EffectConsumer<'a> {
    effects: &'a mut Vec<Effect>,
}
impl<'a> EffectConsumer<'a> {
    pub fn new(effects: &'a mut Vec<Effect>) -> Self {
        effects.clear();
        Self { effects }
    }
}
impl<'a> EffectConsumerTrait for EffectConsumer<'a> {
    fn schedule_effect(&mut self, effect: Effect) -> Result<(), execute_status::FailureStatus> {
        self.effects.push(effect);
        Ok(())
    }
}

/// Runs one algorithm instance, maintaining per-actor reconciliation and effect buffers.
///
/// Execution model:
/// - create a fresh intent consumer borrowing the actor’s order buffer (idx starts at 0)
/// - create a fresh effect consumer borrowing the actor’s effect buffer (clears old effects)
/// - call `algo.execute(...)`
/// - if the algorithm returns Ok(_), handle effects
/// - if it returns `None`, the actor is considered failed and the caller should terminate it
pub struct Actor<Algo>
where
    Algo: ActorAlgo,
{
    id: u32,
    algo: Box<Algo>,
    reconciliator: Reconciliator,
    effect_handler: EffectHandler,
}

impl<Algo> Actor<Algo>
where
    Algo: ActorAlgo,
{
    /// Create a new actor instance running the given algorithm.
    ///
    /// The reconciliator is sized according to `algo.num_intents()`.
    /// The effect handler starts with an empty effect buffer as the number of effects is dynamic.
    pub fn new(id: u32, algo: Box<Algo>) -> Self {
        let num_intents = algo.num_intents();
        let reconciliator = Reconciliator::new(num_intents);
        let effect_handler = EffectHandler::new();
        Self {
            id,
            algo,
            reconciliator,
            effect_handler,
        }
    }

    /// Get the actor’s unique ID.
    fn actor_id(&self) -> u32 {
        self.id
    }

    /// Get the contracts this actor’s algorithm depends on.
    ///
    /// This is used by the controller to subscribe to market data updates.
    fn contracts(&self) -> Vec<Contract> {
        self.algo.contract_deps()
    }

    /// Execute the actor’s algorithm for one tick.
    ///
    /// Returns `Ok(execute_status::Success)` on success, or `Err(execute_status::Failure)`
    /// if the algorithm failed.
    ///
    /// Intents are handled by the reconciliator updating the order buffer in-place.
    /// Effects are collected in the effect handler and processed after execution.
    fn execute(&mut self, context: &ActorExecutionContext) -> execute_status::ExecuteResult {
        let effect_handler = &mut self.effect_handler;
        let reconciliator = &mut self.reconciliator;
        self.algo
            .execute(
                context,
                &mut reconciliator.intent_consumer(),
                &mut effect_handler.effect_consumer(),
            )
            .and_then(|_| effect_handler.handle_effects())
    }
}

/// Trait object interface for actors.
///
/// The actor trait is the view presented to the actor controller and type-erases the
/// underlying algorithm implementation in actors.
pub trait ActorTrait {
    /// Get the actor’s unique ID.
    fn actor_id(&self) -> u32;

    /// Get the contracts this actor’s algorithm depends on.
    ///
    /// The controller uses this to subscribe to market data updates.
    fn contracts(&self) -> Vec<Contract>;

    /// Execute the actor’s algorithm for one tick.
    ///
    /// Returns `Ok(execute_status::Success)` on success, or `Err(execute_status::Failure)`
    /// if the algorithm failed.
    fn execute(&mut self, ctx: &ActorExecutionContext) -> execute_status::ExecuteResult;
}

impl<Algo> ActorTrait for Actor<Algo>
where
    Algo: ActorAlgo,
{
    fn actor_id(&self) -> u32 {
        Actor::actor_id(self)
    }
    fn contracts(&self) -> Vec<Contract> {
        Actor::contracts(self)
    }
    fn execute(&mut self, ctx: &ActorExecutionContext) -> execute_status::ExecuteResult {
        Actor::execute(self, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Minimal, immutable mock algorithm for testing `Actor`.
    ///
    /// Properties:
    /// - Emits exactly `n` intents on every successful execution
    /// - Intents are either:
    ///   - all `NoIntent` (if `intents` is empty), or
    ///   - exactly the provided `intents` (length must be `n`)
    /// - Emits the same effects on every execution
    /// - Can be configured to fail on a specific run index
    ///
    /// The only mutable state is the execution counter (`run`).
    struct MockAlgo {
        /// Number of intents produced per execution.
        n: usize,

        /// Contract dependencies.
        deps: Vec<Contract>,

        /// Immutable intent template.
        ///
        /// If empty, `n` `NoIntent`s are emitted.
        /// Otherwise, length must be exactly `n`.
        intents: Vec<block_traits::Intent>,

        /// Immutable effect template.
        effects: Vec<Effect>,

        /// Counts how many times `execute` has been called.
        run: Cell<u32>,

        /// If set, execution returns `None` on this run index.
        fail_on: Option<u32>,
    }

    impl MockAlgo {
        fn new(
            n: usize,
            deps: Vec<Contract>,
            intents: Vec<block_traits::Intent>,
            effects: Vec<Effect>,
        ) -> Self {
            let intents = if intents.is_empty() {
                vec![Intent::NoIntent; n]
            } else {
                assert_eq!(
                    intents.len(),
                    n,
                    "MockAlgo intents length must equal num_intents()"
                );
                intents
            };

            Self {
                n,
                deps,
                intents,
                effects,
                run: Cell::new(0),
                fail_on: None,
            }
        }

        fn fail_on(mut self, k: u32) -> Self {
            self.fail_on = Some(k);
            self
        }
    }

    impl block_traits::ContractDeps for MockAlgo {
        fn contract_deps(&self) -> Vec<Contract> {
            self.deps.clone()
        }
    }

    impl<'a> ExecuteTrait<ActorExecutionContext, ReconcileIntentConsumer<'a>, EffectConsumer<'a>>
        for MockAlgo
    {
        fn num_intents(&self) -> usize {
            self.n
        }

        fn execute(
            &self,
            _context: &ActorExecutionContext,
            intent_consumer: &mut ReconcileIntentConsumer<'a>,
            effect_consumer: &mut EffectConsumer<'a>,
        ) -> execute_status::ExecuteResult {
            // Count number of executions
            let run = self.run.get();
            self.run.set(run + 1);

            // Fail if configured to do so
            if self.fail_on == Some(run) {
                return Err(execute_status::Failure);
            }

            // Emit intents
            for intent in &self.intents {
                intent_consumer.consume(intent)?;
            }

            // Emit effects
            for effect in &self.effects {
                effect_consumer.schedule_effect(effect.clone())?;
            }

            Ok(execute_status::Success)
        }
    }

    // ──────────────────────────────── tests ────────────────────────────────

    #[test]
    fn actor_sizes_reconciliator_from_num_intents() {
        let algo = Box::new(MockAlgo::new(3, vec![], vec![], vec![]));
        let actor = Actor::new(1, algo);

        assert_eq!(actor.reconciliator.orders.len(), 3);
    }

    #[test]
    fn actor_forwards_id_and_contracts() {
        let c1 = Contract::new("A");
        let c2 = Contract::new("B");

        let algo = Box::new(MockAlgo::new(
            0,
            vec![c1.clone(), c2.clone()],
            vec![],
            vec![],
        ));
        let actor = Actor::new(42, algo);

        assert_eq!(actor.actor_id(), 42);
        assert_eq!(actor.contracts(), vec![c1, c2]);
    }

    #[test]
    fn emits_no_intents_when_intents_template_is_empty() {
        let algo = Box::new(MockAlgo::new(2, vec![], vec![], vec![]));
        let mut actor = Actor::new(1, algo);

        let ctx = ActorExecutionContext::new(0);
        assert!(actor.execute(&ctx).is_ok());
    }

    #[test]
    fn collects_effects_from_algo() {
        let effects = vec![Effect::timer(10), Effect::suspend(), Effect::terminate()];

        let algo = Box::new(MockAlgo::new(1, vec![], vec![], effects.clone()));
        let mut actor = Actor::new(1, algo);

        let ctx = ActorExecutionContext::new(0);
        actor.execute(&ctx).unwrap();

        assert_eq!(actor.effect_handler.effects, effects);
    }

    #[test]
    fn effect_buffer_is_cleared_between_ticks() {
        let effects = vec![Effect::suspend()];
        let algo = Box::new(MockAlgo::new(1, vec![], vec![], effects));
        let mut actor = Actor::new(1, algo);

        let ctx = ActorExecutionContext::new(0);

        actor.execute(&ctx).unwrap();
        assert_eq!(actor.effect_handler.effects.len(), 1);

        actor.execute(&ctx).unwrap();
        assert_eq!(actor.effect_handler.effects.len(), 1);
    }

    #[test]
    fn execute_returns_none_when_algo_fails() {
        let algo = Box::new(MockAlgo::new(1, vec![], vec![], vec![]).fail_on(0));
        let mut actor = Actor::new(1, algo);

        let ctx = ActorExecutionContext::new(0);
        assert!(actor.execute(&ctx).is_err());
    }
}
