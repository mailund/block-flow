use trade_types::{Contract, Price, Quantity, Side};

/// This is a mock of outbound orders
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Order {
    #[default]
    NoOrder,
    New {
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    },
    Cancel {
        contract: Contract,
    },
}
