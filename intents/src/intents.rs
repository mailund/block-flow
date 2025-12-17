use super::slots::SlotId;
use trade_types::{Contract, Price, Quantity, Side};

#[derive(Clone, Debug)]
pub struct NoIntent {
    pub id: SlotId,
}
impl NoIntent {
    pub fn new(slot_id: SlotId) -> Self {
        NoIntent { id: slot_id }
    }
}

#[derive(Clone, Debug)]
pub struct PlaceIntent {
    pub id: SlotId,
    pub contract: Contract,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

impl PlaceIntent {
    pub fn new(
        id: SlotId,
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        PlaceIntent {
            id,
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
impl Intent {
    pub fn no_intent(slot_id: SlotId) -> Self {
        Intent::NoIntent(NoIntent::new(slot_id))
    }
    pub fn place_intent(
        id: SlotId,
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Intent::Place(PlaceIntent::new(id, contract, side, price, quantity))
    }
}
