use super::*;
use intents::*;
use trade_types::*;

make_defaults!(state, output);

#[input]
pub struct Input {
    pub should_execute: bool,
}

#[init_params]
pub struct InitParams {
    pub contract: Contract,
}

#[block(intents = OneIntent)]
pub struct SimpleOrderBlock {
    pub block_id: u32,
    contract: Contract,
}

impl BlockSpec for SimpleOrderBlock {
    fn block_id(&self) -> u32 {
        self.block_id
    }

    fn new_from_init_params(params: &InitParams) -> Self {
        SimpleOrderBlock {
            block_id: 0,
            contract: params.contract.clone(),
        }
    }

    fn init_state(&self) -> State {
        State
    }

    fn execute(
        &self,
        _context: &ExecutionContext,
        input: Input,
        _state: &State,
    ) -> (Output, State, Self::Intents) {
        let intent = if input.should_execute {
            Intent::place_intent(
                SlotId::new(self.block_id, 0),
                self.contract.clone(),
                Side::Buy,
                Cents(100).into(),
                Kw(1).into(),
            )
        } else {
            Intent::no_intent(SlotId::new(self.block_id, 0))
        };

        (Output, State, Self::Intents::new([intent]))
    }
}
