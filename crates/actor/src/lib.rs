use std::fmt::Debug;

use trade_types::*;

mod actor;
mod controller;
pub use actor::Actor;
pub use controller::ActorController;

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

/// Mock delta
#[derive(Debug)]
pub struct Delta(pub Contract);
