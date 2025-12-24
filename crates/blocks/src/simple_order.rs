use super::*;
use block_traits::BlockSpec;
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
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

#[block(intents = OneIntent)]
pub struct SimpleOrderBlock {
    pub block_id: u32,
    contract: Contract,
    side: Side,
    price: Price,
    quantity: Quantity,
}

impl SimpleOrderBlock {
    fn place_intent(&self) -> Intent {
        Intent::place_intent(
            self.contract.clone(),
            self.side.clone(),
            self.price.clone(),
            self.quantity.clone(),
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

    fn new_from_init_params(
        InitParams {
            contract,
            side,
            price,
            quantity,
        }: &InitParams,
    ) -> Self {
        SimpleOrderBlock {
            block_id: 0,
            contract: contract.clone(),
            side: side.clone(),
            price: price.clone(),
            quantity: quantity.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_defaults_creates_unit_state_and_output() {
        let _state = State;
        let _out = Output;

        let _state2: State = Default::default();
        let _out2: Output = Default::default();

        assert_eq!(core::mem::size_of::<State>(), 0);
        assert_eq!(core::mem::size_of::<Output>(), 0);
    }

    fn test_params() -> (Contract, Side, Price, Quantity, InitParams) {
        let contract = Contract::new("TEST");
        let side = Side::Buy;
        let price = Price::from(Cents(12345));
        let quantity = Quantity::from(Kw(1000));

        let params = InitParams {
            contract: contract.clone(),
            side: side.clone(),
            price: price.clone(),
            quantity: quantity.clone(),
        };

        (contract, side, price, quantity, params)
    }

    fn test_block() -> (Contract, Side, Price, Quantity, SimpleOrderBlock) {
        let (contract, side, price, quantity, _params) = test_params();

        let block = SimpleOrderBlock {
            block_id: 1,
            contract: contract.clone(),
            side: side.clone(),
            price: price.clone(),
            quantity: quantity.clone(),
        };

        (contract, side, price, quantity, block)
    }

    #[test]
    fn new_from_init_params_sets_block_id_default() {
        let (_contract, _side, _price, _quantity, params) = test_params();

        let block = SimpleOrderBlock::new_from_init_params(&params);

        assert_eq!(block.block_id(), 0);
    }

    #[test]
    fn init_state_returns_default_state() {
        let (contract, side, price, quantity, _params) = test_params();

        let block = SimpleOrderBlock {
            block_id: 42,
            contract,
            side,
            price,
            quantity,
        };

        let state = block.init_state();
        assert!(matches!(state, State));
    }

    #[test]
    fn execute_with_should_execute_true_returns_place_intent() {
        let (contract, side, price, quantity, block) = test_block();

        let ctx = ExecutionContext { time: 0 };
        let state = State;

        let (_out, _state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_execute: true,
                },
                &state,
            )
            .unwrap();

        let intents_arr = intents.as_slice();
        assert_eq!(intents_arr.len(), 1);

        match &intents_arr[0] {
            Intent::Place(place) => {
                assert_eq!(&place.contract, &contract);
                assert_eq!(&place.side, &side);
                assert_eq!(&place.price, &price);
                assert_eq!(&place.quantity, &quantity);
            }
            Intent::NoIntent(_) => {
                panic!("Expected Place intent, got NoIntent");
            }
        }
    }

    #[test]
    fn execute_with_should_execute_false_returns_no_intent() {
        let (_contract, _side, _price, _quantity, block) = test_block();

        let ctx = ExecutionContext { time: 0 };
        let state = State;

        let (_out, _state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_execute: false,
                },
                &state,
            )
            .unwrap();

        let intents_arr = intents.as_slice();
        assert_eq!(intents_arr.len(), 1);

        match &intents_arr[0] {
            Intent::NoIntent(_) => {}
            Intent::Place(_) => {
                panic!("Expected NoIntent, got Place");
            }
        }
    }

    #[test]
    fn intents_helper_matches_execute_behavior() {
        let (_contract, _side, _price, _quantity, block) = test_block();

        let t = block.intents(true).as_slice().to_vec();
        let f = block.intents(false).as_slice().to_vec();

        assert_eq!(t.len(), 1);
        assert_eq!(f.len(), 1);

        assert!(matches!(t[0], Intent::Place(_)));
        assert!(matches!(f[0], Intent::NoIntent(_)));
    }
}
