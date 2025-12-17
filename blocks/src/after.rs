use super::*;

// Default (empty struct) Input and State
make_defaults!(input, state,);

#[output]
pub struct Output {
    pub is_after: bool,
}

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

    #[execute]
    fn execute(&self, context: &ExecutionContext) -> Output {
        let is_after = context.time > self.time;
        Output { is_after }
    }
}
