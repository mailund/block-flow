use super::*;
use block_traits::intents;
use block_traits::ExecuteTrait;
use std::cell::{Ref, RefCell};
use trade_types::Contract;

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

/// A mock actor.
pub struct Actor {
    id: u32,

    /// The block encapsulated by this actor.
    /// A block can be a simple block or a composite block,
    /// so in practice the block is usually an execution plan
    /// containing multiple blocks.
    block: Box<dyn ExecuteTrait<ActorExecutionContext>>, // FIXME: We could use generics here to better control the instruction set

    /// Array of the orders the actor can emit.
    /// RefCell for interior mutability.
    orders: RefCell<Vec<Order>>,
}

impl Actor {
    pub fn new(id: u32, block: Box<dyn ExecuteTrait<ActorExecutionContext>>) -> Self {
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

    fn reconcile_intents(&self, intents: &[intents::Intent]) -> Ref<'_, [Order]> {
        let mut orders = self.orders.borrow_mut();

        // Resize -- the intents will always have the same length and we
        // *could* allocate this up front if we could compute this length
        // (which we almost can; we can for all Intents and for execution
        // plans we could compute it upon creation). But I haven't bothered
        // yet.
        orders.resize(intents.len(), Order::NoOrder);

        // Fill in-place
        for (i, intent) in intents.iter().enumerate() {
            orders[i] = match &intent {
                intents::Intent::NoIntent(_) => Order::NoOrder,
                intents::Intent::Place(place) => Order::New {
                    contract: place.contract.clone(),
                    side: place.side.clone(),
                    price: place.price.clone(),
                    quantity: place.quantity.clone(),
                },
            };
        }

        // Return the updated orders as a Ref
        let orders = self.orders.borrow();
        Ref::map(orders, |o| o.as_slice())
    }

    /// Perform a tick of the actor, given the execution context.
    /// The tick will execute the underlying block and reconcile the
    /// resulting intents into orders. The function then returns
    /// the new orders or None if the block could not execute.
    pub(crate) fn tick(&'_ self, context: &ActorExecutionContext) -> Option<Ref<'_, [Order]>> {
        let intents = self.block.execute(context)?;
        let orders = self.reconcile_intents(&intents);
        Some(orders)
    }
}
