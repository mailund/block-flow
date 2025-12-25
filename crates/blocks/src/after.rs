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
    fn execute<C: ExecutionContextTrait>(&self, context: &C) -> Output {
        let is_after = context.time() > self.time;
        Output { is_after }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trade_types::{Contract, OrderBook};

    pub struct ExecutionContext {
        pub time: u64,
    }

    impl ExecutionContextTrait for ExecutionContext {
        fn time(&self) -> u64 {
            self.time
        }
        fn get_order_book(&self, _contract: &Contract) -> Option<OrderBook> {
            // Mock implementation
            Some(OrderBook {})
        }
    }

    fn ctx(time: u64) -> ExecutionContext {
        ExecutionContext { time }
    }

    #[test]
    fn make_defaults_creates_unit_input_and_state() {
        // These compile only if the macro generated them.
        let _input = Input;
        let _state = State;

        // And they should be Default if you’re relying on defaults elsewhere.
        let _input2: Input = Default::default();
        let _state2: State = Default::default();

        // Optional sanity check: unit structs are zero-sized.
        assert_eq!(core::mem::size_of::<Input>(), 0);
        assert_eq!(core::mem::size_of::<State>(), 0);
    }

    #[test]
    fn new_from_init_params_sets_time_and_block_id_default() {
        let params = InitParams { time: 10 };
        let block = AfterBlock::new_from_init_params(&params);

        // Your impl sets block_id = 0 in new_from_init_params
        assert_eq!(block.block_id(), 0);
    }

    #[test]
    fn execute_returns_output_and_defaults_for_missing_parts_before_time() {
        let params = InitParams { time: 10 };
        let block = AfterBlock::new_from_init_params(&params);

        let context = ctx(9);

        // Call the *full* signature (what #[execute] generates).
        let (out, state_out, _intents) = block.execute(&context, Input, &State).unwrap();

        assert!(!out.is_after);
        assert!(matches!(state_out, State)); // default-filled (unit)
    }

    #[test]
    fn execute_returns_output_and_defaults_for_missing_parts_after_time() {
        let params = InitParams { time: 10 };
        let block = AfterBlock::new_from_init_params(&params);

        let context = ctx(11);

        let (out, state_out, _intents) = block.execute(&context, Input, &State).unwrap();

        assert!(out.is_after);
        assert!(matches!(state_out, State));
    }

    #[test]
    fn execute_is_false_when_equal() {
        let params = InitParams { time: 10 };
        let block = AfterBlock::new_from_init_params(&params);

        let context = ctx(10);

        let (out, _state_out, _intents) = block.execute(&context, Input, &State).unwrap();

        assert!(!out.is_after);
    }
}
