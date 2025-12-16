use super::*;
use intents::ZeroIntents;

#[input]
pub struct Input {
    pub should_delete: bool,
}

#[output]
pub struct Output;

#[state]
pub struct State;

#[init_params]
pub struct InitParams;

#[block]
pub struct DeleteBlock {
    pub block_id: u32,
}

impl BlockSpec for DeleteBlock {
    fn block_id(&self) -> u32 {
        self.block_id
    }

    fn new_from_init_params(_params: &InitParams) -> Self {
        DeleteBlock { block_id: 0 }
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
        if input.should_delete {
            // In a real implementation, this would trigger deletion logic.
            println!("DeleteBlock: Deletion triggered.");
        } else {
            println!("DeleteBlock: No deletion.");
        }
        let output = Output;
        (output, State, ZeroIntents::new())
    }
}
