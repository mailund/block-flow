use super::order_book::OrderBookTrait;
use trade_types::*;

pub trait ExecutionContextTrait {
    type OrderBook: OrderBookTrait;

    fn time(&self) -> u64;
    fn get_order_book(&self, contract: &Contract) -> Option<Self::OrderBook>;
    fn get_position(&self, block_id: u32, contract: &Contract) -> Option<Quantity>;
}
