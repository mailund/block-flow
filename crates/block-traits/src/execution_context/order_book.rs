use trade_types::*;

/// This is a mock trait for what an order book could look like.
pub trait OrderBookTrait {
    fn top_of_side(&self, _side: Side) -> Option<Price>;
}
