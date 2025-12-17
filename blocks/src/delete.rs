use super::*;

make_defaults!(output, state, init_params);

#[input]
pub struct Input {
    pub should_delete: bool,
}

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

    #[execute]
    fn execute(&self, input: Input) {
        if input.should_delete {
            // In a real implementation, this would trigger deletion logic.
            println!("DeleteBlock: Deletion triggered.");
        } else {
            println!("DeleteBlock: No deletion.");
        }
    }
}
