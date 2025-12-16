use super::*;
use trade_types::Contract;

#[input]
pub struct Input {
    pub should_execute: bool,
}

#[output]
pub struct Output;

#[state]
pub struct State;

#[init_params]
pub struct InitParams {
    pub contract: Contract,
}

#[block]
pub struct SimpleOrderBlock {
    contract: Contract,
}

impl BlockSpec for SimpleOrderBlock {
    fn new_from_init_params(params: &InitParams) -> Self {
        SimpleOrderBlock {
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
    ) -> (Output, State) {
        if input.should_execute {
            // In a real implementation, this would trigger order placement logic.
            println!(
                "SimpleOrderBlock: Placing order for contract {:?}.",
                self.contract
            );
        }
        (Output, State)
    }
}
