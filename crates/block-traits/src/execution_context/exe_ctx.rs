use super::order_book::OrderBookTrait;
use trade_types::*;

pub trait ExecutionContextTrait {
    type OrderBook: OrderBookTrait;

    fn time(&self) -> u64;
    fn get_order_book(&self, contract: &Contract) -> Option<Self::OrderBook>;
}
