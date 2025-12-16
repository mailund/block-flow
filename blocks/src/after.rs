use super::*;

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
    time: u64,
}

impl BlockSpec for AfterBlock {
    fn new_from_init_params(params: &InitParams) -> Self {
        AfterBlock { time: params.time }
    }

    fn init_state(&self) -> State {
        State
    }

    fn execute(
        &self,
        context: &ExecutionContext,
        _input: Input,
        _state: &State,
    ) -> (Output, State) {
        let is_after = context.time > self.time;
        let output = Output { is_after };
        (output, State)
    }
}
