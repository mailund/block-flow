use super::*;

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
pub struct DeleteBlock;

impl BlockSpec for DeleteBlock {
    fn new_from_init_params(_params: &InitParams) -> Self {
        DeleteBlock
    }

    fn init_state(&self) -> State {
        State
    }

    fn execute(
        &self,
        _context: &ExecutionContext,
        input: Input,
        _state: &State,
    ) -> (Output, State) {
        if input.should_delete {
            // In a real implementation, this would trigger deletion logic.
            println!("DeleteBlock: Deletion triggered.");
        } else {
            println!("DeleteBlock: No deletion.");
        }
        let output = Output;
        (output, State)
    }
}
