use super::*;
use block_traits::{ExecuteTrait, Intent, IntentConsumerTrait};
use trade_types::Contract;

/// "Type alias" (for a trait) for algorithms an actor can run.
pub trait ActorAlgo: ExecuteTrait<ActorExecutionContext, Reconcile> {}
impl<T> ActorAlgo for T where T: ExecuteTrait<ActorExecutionContext, Reconcile> {}

/// Reconciliation book-keeping (mock for now) and the consumer trait
/// for invoking reconciliation on intents produced by the actor's block.
pub struct Reconcile {
    pub(super) orders: Vec<Order>,
    idx: usize,
}
impl Reconcile {
    pub fn new(no_intents: usize) -> Self {
        Self {
            orders: vec![Order::NoOrder; no_intents],
            idx: 0,
        }
    }
    pub fn reconcile(&mut self) -> &mut Reconcile {
        self.idx = 0;
        self
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
}

/// Implement the IntentConsumerTrait for Reconcile to process intents.
/// This gives us a callback that is invoked after each block's execution
/// where we can reconsile and push order updates out.
impl IntentConsumerTrait for Reconcile {
    fn consume(&mut self, intent: &Intent) {
        self.orders[self.idx] = self.process_intent(&self.orders[self.idx], intent);
        self.idx += 1;
    }
}

pub struct Actor<Algo>
where
    Algo: ActorAlgo,
{
    id: u32,
    algo: Box<Algo>,
    reconcile: Reconcile,
}

impl<Algo> Actor<Algo>
where
    Algo: ActorAlgo,
{
    pub fn new(id: u32, algo: Box<Algo>) -> Self {
        let no_intents = algo.no_intents();
        let reconcile = Reconcile::new(no_intents);
        Self {
            id,
            algo,
            reconcile,
        }
    }

    fn actor_id(&self) -> u32 {
        self.id
    }

    fn contracts(&self) -> Vec<Contract> {
        self.algo.contract_deps()
    }

    fn execute(&mut self, context: &ActorExecutionContext) -> Option<()> {
        self.algo.execute(context, self.reconcile.reconcile())
    }
}

pub trait ActorTrait {
    /// Id of the actor / algorithm
    fn actor_id(&self) -> u32;
    /// Contracts registered for ticks upon initialization
    fn contracts(&self) -> Vec<Contract>;
    /// Perform a tick for the actor, returning any orders generated.
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
