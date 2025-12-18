mod block_intents;
mod intents;
mod slots;

pub use block_intents::{
    BlockIntents, FiveIntents, FourIntents, OneIntent, ThreeIntents, TwoIntents, ZeroIntents,
};
pub use intents::*;
pub use slots::SlotId;

pub struct SlotIntent {
    pub slot_id: SlotId,
    pub intent: Intent,
}
impl SlotIntent {
    pub fn new(slot_id: SlotId, intent: Intent) -> Self {
        SlotIntent { slot_id, intent }
    }
}
