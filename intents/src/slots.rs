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
    pub fn set_slot_id(&mut self, slot_index: u32) {
        self.slot_index = slot_index;
    }
}
