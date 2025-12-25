use trade_types::*;

pub trait ExecutionContextTrait {
    fn time(&self) -> u64;
    fn get_order_book(&self, contract: &Contract) -> Option<OrderBook>;
}
