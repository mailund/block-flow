use super::*;
use block_traits::{ContractDeps, ExecuteTrait, Intent, IntentConsumerTrait};
use trade_types::Contract;

/// Reconciliation book-keeping (mock for now) and the consumer trait
/// for invoking reconciliation on intents produced by the actor's block.
pub struct Reconcile {
    orders: Vec<Order>,
    idx: usize,
}

/// Implement the IntentConsumerTrait for Reconcile to process intents.
/// This gives us a callback that is invoked after each block's execution
/// where we can reconsile and push order updates out.
impl IntentConsumerTrait for Reconcile {
    fn consume(&mut self, intent: &Intent) {
        // The implementation here is still a mock
        if self.idx >= self.orders.len() {
            self.orders.push(Order::NoOrder);
        }

        self.orders[self.idx] = match intent {
            Intent::NoIntent(_) => Order::NoOrder,
            Intent::Place(place) => Order::New {
                contract: place.contract.clone(),
                side: place.side.clone(),
                price: place.price.clone(),
                quantity: place.quantity.clone(),
            },
        };

        self.idx += 1;
    }
}

pub struct Actor<B> {
    id: u32,
    block: B,
    reconcile: Reconcile,
}

impl<B> Actor<B>
where
    B: ExecuteTrait<ActorExecutionContext, Reconcile>,
{
    pub fn new(id: u32, block: B) -> Self {
        Self {
            id,
            block,
            reconcile: Reconcile {
                orders: vec![],
                idx: 0,
            },
        }
    }

    pub fn actor_id(&self) -> u32 {
        self.id
    }

    pub fn contracts(&self) -> Vec<Contract> {
        self.block.contract_deps()
    }

    pub(crate) fn tick(&mut self, context: &ActorExecutionContext) -> Option<()> {
        self.block.execute(context, &mut self.reconcile)
    }
}

pub trait ActorTrait {
    /// Id of the actor / algorithm
    fn actor_id(&self) -> u32;
    /// Contracts registered for ticks upon initialization
    fn contracts(&self) -> Vec<Contract>;
    /// Perform a tick for the actor, returning any orders generated.
    fn tick(&mut self, ctx: &ActorExecutionContext) -> Option<()>;
}

impl<B> ActorTrait for Actor<B>
where
    B: ContractDeps,
    for<'a> B: ExecuteTrait<ActorExecutionContext, Reconcile>,
{
    fn actor_id(&self) -> u32 {
        Actor::actor_id(self)
    }
    fn contracts(&self) -> Vec<Contract> {
        Actor::contracts(self)
    }
    fn tick(&mut self, ctx: &ActorExecutionContext) -> Option<()> {
        Actor::tick(self, ctx)
    }
}
