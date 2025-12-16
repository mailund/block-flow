use super::*;

#[input]
pub struct Input {
    pub time: u64,
}

#[output]
pub struct Output {
    pub is_after: bool,
}

#[state]
pub struct State;

#[init_params]
pub struct InitParams;

#[block]
pub struct AfterBlock;

impl BlockSpec for AfterBlock {
    fn new_from_init_params(_params: &InitParams) -> Self {
        AfterBlock
    }

    fn init_state(&self) -> State {
        State
    }

    fn execute(&self, context: &ExecutionContext, input: Input, _state: &State) -> (Output, State) {
        let is_after = context.time > input.time;
        let output = Output { is_after };
        (output, State)
    }
}
