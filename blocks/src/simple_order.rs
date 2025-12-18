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
    // FIXME: Put side, price, quantity here as well.
}

#[block(intents = OneIntent)]
pub struct SimpleOrderBlock {
    pub block_id: u32,
    contract: Contract,
}

impl SimpleOrderBlock {
    fn place_intent(&self) -> Intent {
        Intent::place_intent(
            self.contract.clone(),
            Side::Buy,
            Cents(100).into(),
            Kw(1).into(),
        )
    }

    fn no_intent(&self) -> Intent {
        Intent::no_intent()
    }

    fn intents(&self, execute: bool) -> OneIntent {
        if execute {
            OneIntent::new([self.place_intent()])
        } else {
            OneIntent::new([self.no_intent()])
        }
    }
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

    #[execute]
    fn execute(&self, input: Input) -> Self::Intents {
        self.intents(input.should_execute)
    }
}
