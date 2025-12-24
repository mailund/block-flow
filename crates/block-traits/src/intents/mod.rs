mod block_intents;
mod intent_types;
mod slots;

pub use block_intents::*;
pub use intent_types::*;
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

#[cfg(test)]
mod tests {
    use super::*;
    use trade_types::*;

    #[test]
    fn test_slot_intents_have_block_id_and_slot_index() {
        let slot_intent = super::SlotIntent::new(
            super::SlotId::new(86, 42),
            super::Intent::place_intent(
                Contract::new("TEST"),
                Side::Buy,
                Cents(100).into(),
                Kw(1).into(),
            ),
        );
        assert_eq!(slot_intent.slot_id.block_id, 86);
        assert_eq!(slot_intent.slot_id.slot_index, 42);
    }
}
