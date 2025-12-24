use trade_types::*;

/// Execution context passed to blocks during execution.
pub struct ExecutionContext {
    pub time: u64,
}

impl ExecutionContext {
    /// Create a new execution context with the given time.
    pub fn new(time: u64) -> Self {
        Self { time }
    }

    pub fn get_order_book(&self, _contract: &Contract) -> Option<OrderBook> {
        // Mock implementation
        Some(OrderBook {})
    }
}
