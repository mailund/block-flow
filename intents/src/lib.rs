use trade_types::{Contract, Price, Quantity, Side};

#[derive(Clone, Debug)]
pub struct SlotId {
    // Dummy impl.
    pub block_id: u32,
    pub slot_index: u32,
}
impl SlotId {
    pub fn new(block_id: u32, slot_index: u32) -> Self {
        SlotId {
            block_id,
            slot_index,
        }
    }
}

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

#[derive(Clone, Debug)]
pub enum BlockEnum {
    Zero,
    One(Intent),
    Two(Intent, Intent),
    Three(Intent, Intent, Intent),
    Four(Intent, Intent, Intent, Intent),
}
impl BlockEnum {
    pub fn intents(self) -> Vec<Intent> {
        match self {
            BlockEnum::Zero => vec![],
            BlockEnum::One(i1) => vec![i1],
            BlockEnum::Two(i1, i2) => vec![i1, i2],
            BlockEnum::Three(i1, i2, i3) => vec![i1, i2, i3],
            BlockEnum::Four(i1, i2, i3, i4) => vec![i1, i2, i3, i4],
        }
    }
}

pub trait BlockIntents {
    fn intents(&self) -> BlockEnum;
}

#[derive(Clone, Debug)]
pub struct ZeroIntents;
impl ZeroIntents {
    pub fn new() -> Self {
        ZeroIntents
    }
}
impl BlockIntents for ZeroIntents {
    fn intents(&self) -> BlockEnum {
        BlockEnum::Zero
    }
}

#[derive(Clone, Debug)]
pub struct OneIntent(pub Intent);
impl OneIntent {
    pub fn new(intent: Intent) -> Self {
        OneIntent(intent)
    }
}
impl BlockIntents for OneIntent {
    fn intents(&self) -> BlockEnum {
        BlockEnum::One(self.0.clone())
    }
}

#[derive(Clone, Debug)]
pub struct TwoIntents(pub Intent, pub Intent);
impl TwoIntents {
    pub fn new(intent1: Intent, intent2: Intent) -> Self {
        TwoIntents(intent1, intent2)
    }
}
impl BlockIntents for TwoIntents {
    fn intents(&self) -> BlockEnum {
        BlockEnum::Two(self.0.clone(), self.1.clone())
    }
}
