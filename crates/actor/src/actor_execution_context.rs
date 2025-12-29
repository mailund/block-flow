// This module is all mock for now

use block_traits::execution_context::{ExecutionContextTrait, OrderBookTrait};
use trade_types::{Cents, Contract, Price, Side};

pub struct OrderBook;

impl OrderBookTrait for OrderBook {
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
    fn get_position(&self, _block_id: u32, _contract: &Contract) -> Option<trade_types::Quantity> {
        // mock position
        None
    }
}
