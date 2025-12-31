use super::*;
use block_traits::{Effect, EffectConsumerTrait, ExecuteTrait, Intent, IntentConsumerTrait};
use trade_types::Contract;

/// "Type alias" (for a trait) for algorithms an actor can run.
pub trait ActorAlgo:
    for<'a> ExecuteTrait<ActorExecutionContext, ReconcileIntentConsumer<'a>, EffectConsumer<'a>>
{
}
impl<T> ActorAlgo for T where
    T: for<'a> ExecuteTrait<ActorExecutionContext, ReconcileIntentConsumer<'a>, EffectConsumer<'a>>
{
}

struct Reconciliator {
    orders: Vec<Order>,
}
impl Reconciliator {
    pub fn new(size: usize) -> Self {
        Self {
            orders: vec![Order::default(); size],
        }
    }
    pub fn intent_consumer(&mut self) -> ReconcileIntentConsumer<'_> {
        ReconcileIntentConsumer::new(&mut self.orders)
    }
}

/// Reconciliation book-keeping (mock for now) and the consumer trait
/// for invoking reconciliation on intents produced by the actor's block.
pub struct ReconcileIntentConsumer<'a> {
    orders: &'a mut [Order],
    idx: usize,
}
impl<'a> ReconcileIntentConsumer<'a> {
    /// Create a new Reconcile intent consumer with a mutable slice of orders.
    /// This can be passed to an actor algorithm's execute method to process intents.
    pub fn new(orders: &'a mut [Order]) -> Self {
        Self { orders, idx: 0 }
    }
    /// Process an intent into an order, given the previous order state.
    /// This is a mock implementation for now; the real implementation should decide
    /// whether to act on a new intent based on the previous order state.
    fn process_intent(&self, prev_order: &Order, intent: &Intent) -> Order {
        match intent {
            Intent::NoIntent(_) => prev_order.clone(),
            Intent::Place(place) => Order::New {
                contract: place.contract.clone(),
                side: place.side.clone(),
                price: place.price.clone(),
                quantity: place.quantity.clone(),
            },
        }
    }
    /// Process one intent at a time, updating the orders slice based on the intent
    /// and the previous order state. This is a mock and the real implementation should
    /// likely output order updates to an outbound channel or similar.
    fn consume(&mut self, intent: &Intent) {
        self.orders[self.idx] = self.process_intent(&self.orders[self.idx], intent);
        self.idx += 1;
    }
}

/// Implement the IntentConsumerTrait for Reconcile to process intents.
/// This gives us a callback that is invoked after each block's execution
/// where we can reconsile and push order updates out.
impl<'a> IntentConsumerTrait for ReconcileIntentConsumer<'a> {
    fn consume(&mut self, intent: &Intent) {
        ReconcileIntentConsumer::consume(self, intent);
    }
}

struct EffectHandler {
    effects: Vec<Effect>,
}
impl EffectHandler {
    fn new() -> Self {
        let effects = Vec::new();
        Self { effects }
    }
    fn effects(&self) -> &Vec<Effect> {
        &self.effects
    }
    fn effect_consumer(&mut self) -> EffectConsumer<'_> {
        EffectConsumer::new(&mut self.effects)
    }
}

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
    fn schedule_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

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

    fn actor_id(&self) -> u32 {
        self.id
    }

    fn contracts(&self) -> Vec<Contract> {
        self.algo.contract_deps()
    }

    /// Handle effects after execution.
    fn handle_effects(&self) {
        // The effect handler ensures that any new effects produced during execution
        // are collected in the effects() vector and can be processed here.
        for effect in self.effect_handler.effects() {
            println!("Actor {} handling effect: {:?}", self.id, effect);
        }
    }

    /// Executing the algo. Intent and effect handling is done through callbacks.
    /// This enables the actor to output orders as they are produced by the algo.
    /// That way, the actor can respond to intents immediately without waiting
    /// for the entire algo to finish executing.
    /// Effects are collected and handled after execution as long as the algorithm
    /// terminates successfully (i.e., does not return None).
    fn execute(&mut self, context: &ActorExecutionContext) -> Option<()> {
        let mut intent_consumer = self.reconciliator.intent_consumer();
        let mut effect_consumer = self.effect_handler.effect_consumer();
        self.algo
            .execute(context, &mut intent_consumer, &mut effect_consumer)
            .map(|_| self.handle_effects())
    }
}

pub trait ActorTrait {
    /// Id of the actor / algorithm
    fn actor_id(&self) -> u32;
    /// Contracts registered for ticks upon initialization
    fn contracts(&self) -> Vec<Contract>;
    /// Perform a tick for the actor, returning None if the actor failed and should be terminated.
    fn execute(&mut self, ctx: &ActorExecutionContext) -> Option<()>;
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
    fn execute(&mut self, ctx: &ActorExecutionContext) -> Option<()> {
        Actor::execute(self, ctx)
    }
}
