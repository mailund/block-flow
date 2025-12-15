use block_macros::{block, input, output, state};
use block_traits::{BlockSpec, ExecutionContext};
use registry;

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

#[block]
pub struct AfterBlock;

impl BlockSpec for AfterBlock {
    fn init_state(&self) -> State {
        State
    }

    fn execute(&self, context: &ExecutionContext, input: Input, _state: &State) -> (Output, State) {
        let is_after = context.time > input.time;
        let output = Output { is_after };
        (output, State)
    }
}
