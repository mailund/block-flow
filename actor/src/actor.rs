use super::*;
use block_traits::{Block, BlockTrait};
use execution_context::ExecutionContext;
use intents::SlotIntent;
use std::cell::{Ref, RefCell};
use trade_types::Contract;

/// A mock actor.
pub struct Actor {
    /// The block encapsulated by this actor.
    /// A block can be a simple block or a composite block,
    /// so in practice the block is usually an execution plan
    /// containing multiple blocks.
    block: Block,

    /// Array of the orders the actor can emit.
    /// RefCell for interior mutability.
    orders: RefCell<Vec<Order>>,
}

impl Actor {
    /// Create a new actor encapsulating the given block.
    pub fn new(block: Block) -> Self {
        Self {
            block,
            orders: RefCell::new(vec![]),
        }
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn contracts(&self) -> Vec<Contract> {
        self.block.contract_deps()
    }

    fn reconcile_intents(&self, intents: &[SlotIntent]) -> Ref<'_, [Order]> {
        let mut orders = self.orders.borrow_mut();

        // Resize -- the intents will always have the same length and we
        // *could* allocate this up front if we could compute this length
        // (which we almost can; we can for all Intents and for execution
        // plans we could compute it upon creation). But I haven't bothered
        // yet.
        orders.resize(intents.len(), Order::NoOrder);

        // Fill in-place
        for (i, intent) in intents.iter().enumerate() {
            orders[i] = match &intent.intent {
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
    pub fn tick(&'_ self, context: &ExecutionContext) -> Option<Ref<'_, [Order]>> {
        let intents = self.block.execute(context)?;
        let orders = self.reconcile_intents(&intents);
        Some(orders)
    }
}
