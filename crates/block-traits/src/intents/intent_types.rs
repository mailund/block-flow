use std::fmt::Binary;

use trade_types::{Contract, Price, Quantity, Side};

#[derive(Clone, Debug)]
pub struct NoIntent;
impl NoIntent {
    pub fn new() -> Self {
        NoIntent
    }
}
impl Default for NoIntent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct PlaceIntent {
    pub contract: Contract,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

impl PlaceIntent {
    pub fn new(contract: Contract, side: Side, price: Price, quantity: Quantity) -> Self {
        PlaceIntent {
            contract,
            side,
            price,
            quantity,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Intent {
    NoIntent(NoIntent),
    Place(PlaceIntent),
}

pub trait IntentFactory {
    fn no_intent() -> Intent {
        Intent::NoIntent(NoIntent::new())
    }

    fn place_intent(contract: Contract, side: Side, price: Price, quantity: Quantity) -> Intent {
        Intent::Place(PlaceIntent::new(contract, side, price, quantity))
    }
}

impl IntentFactory for Intent {}

// Give the factory to all blocks that load the trait.
impl<B: Binary> IntentFactory for B {}
