mod mock_implementations {
    use block_traits::execution_context::ExecutionContextTrait;
    use trade_types::{Cents, Contract, Price, Side};

    pub struct OrderBook;

    impl block_traits::execution_context::OrderBookTrait for OrderBook {
        fn top_of_side(&self, _side: Side) -> Option<Price> {
            // Dummy implementation
            Some(Price::from(Cents(100)))
        }
    }

    pub struct ActorExecutionContext {
        time: u64,
    }
    impl ActorExecutionContext {
        pub fn new(time: u64) -> Self {
            Self { time }
        }
    }
    impl ExecutionContextTrait for ActorExecutionContext {
        type OrderBook = OrderBook;

        fn time(&self) -> u64 {
            self.time
        }
        fn get_order_book(&self, _contract: &Contract) -> Option<Self::OrderBook> {
            // mock order book
            Some(OrderBook)
        }
        fn get_position(
            &self,
            _block_id: u32,
            _contract: &Contract,
        ) -> Option<trade_types::Quantity> {
            // mock position
            None
        }
    }
}
pub use mock_implementations::ActorExecutionContext;

use super::*;
use std::cell::{Ref, RefCell};
use trade_types::Contract;

use block_traits::{ContractDeps, ExecuteTrait, Intent, IntentConsumerTrait};

/// Reconciliation book-keeping (mock for now) and the consumer trait
/// for invoking reconciliation on intents produced by the actor's block.
pub struct Reconcile<'a> {
    /// Reference to the actor's orders. Thus lifetime is needed (it can't live longer
    /// than the actor itself), and we need mutable access to update the orders, thus
    /// the RefCell (the actor itself is not mutable during tick execution, so we hide
    /// internal mutability here).
    orders: &'a RefCell<Vec<Order>>,
    /// Index into the orders vector to keep track of which order we are processing.
    idx: usize,
}

/// Implement the IntentConsumerTrait for Reconcile to process intents.
/// This gives us a callback that is invoked after each block's execution
/// where we can reconsile and push order updates out.
impl<'a> IntentConsumerTrait for Reconcile<'a> {
    fn consume(&mut self, intent: &Intent) {
        // The implementation here is still a mock
        let mut orders = self.orders.borrow_mut();

        if self.idx >= orders.len() {
            orders.push(Order::NoOrder);
        }

        orders[self.idx] = match intent {
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
    orders: RefCell<Vec<Order>>,
}

impl<B> Actor<B>
where
    B: ContractDeps,
    for<'a> B: ExecuteTrait<ActorExecutionContext, Reconcile<'a>>,
{
    pub fn new(id: u32, block: B) -> Self {
        Self {
            id,
            block,
            orders: RefCell::new(Vec::new()),
        }
    }

    pub fn actor_id(&self) -> u32 {
        self.id
    }

    pub fn contracts(&self) -> Vec<Contract> {
        self.block.contract_deps()
    }

    pub(crate) fn tick<'a>(&'a self, context: &ActorExecutionContext) -> Option<Ref<'a, [Order]>> {
        // Reset orders before each tick
        let mut orders = self.orders.borrow_mut();
        for o in orders.iter_mut() {
            *o = Order::NoOrder;
        }

        let mut consumer = Reconcile {
            orders: &self.orders,
            idx: 0,
        };

        // Execute with the reconcile consumer processing the intents
        self.block.execute(context, &mut consumer)?;

        // Returning the orders -- this is a mock and it should output the orders
        // on IPC channels in a real implementation.
        let orders = self.orders.borrow();
        Some(Ref::map(orders, |o| o.as_slice()))
    }
}

pub trait ActorTrait {
    /// Id of the actor / algorithm
    fn actor_id(&self) -> u32;
    /// Contracts registered for ticks upon initialization
    fn contracts(&self) -> Vec<Contract>;
    /// Perform a tick for the actor, returning any orders generated.
    fn tick(&self, ctx: &ActorExecutionContext) -> Option<Ref<'_, [Order]>>;
}

impl<B> ActorTrait for Actor<B>
where
    B: ContractDeps,
    for<'a> B: ExecuteTrait<ActorExecutionContext, Reconcile<'a>>,
{
    fn actor_id(&self) -> u32 {
        Actor::actor_id(self)
    }
    fn contracts(&self) -> Vec<Contract> {
        Actor::contracts(self)
    }
    fn tick(&self, ctx: &ActorExecutionContext) -> Option<Ref<'_, [Order]>> {
        Actor::tick(self, ctx)
    }
}
