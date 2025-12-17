use intents::ZeroIntents;

use super::*;
use intents::ZeroIntents;

#[input]
pub struct Input;

#[output]
pub struct Output {
    pub is_after: bool,
}

#[state]
pub struct State;

#[init_params]
pub struct InitParams {
    pub time: u64,
}

#[block]
pub struct AfterBlock {
    pub block_id: u32,
    time: u64,
}

impl BlockSpec for AfterBlock {
    fn block_id(&self) -> u32 {
        self.block_id
    }

    fn new_from_init_params(params: &InitParams) -> Self {
        AfterBlock {
            block_id: 0,
            time: params.time,
        }
    }

    fn init_state(&self) -> State {
        State
    }

    fn execute(
        &self,
        context: &ExecutionContext,
        _input: Input,
        _state: &State,
    ) -> (Output, State, Self::Intents) {
        let is_after = context.time > self.time;
        let output = Output { is_after };
        (output, State, ZeroIntents::new())
    }
}
