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

    #[test]
    fn make_defaults_creates_unit_output_state_and_init_params() {
        // Compiles only if the macro generated them
        let _out = Output;
        let _state = State;
        let _params = InitParams;

        // Should be Default if your framework relies on defaults
        let _out2: Output = Default::default();
        let _state2: State = Default::default();
        let _params2: InitParams = Default::default();

        assert_eq!(core::mem::size_of::<Output>(), 0);
        assert_eq!(core::mem::size_of::<State>(), 0);
        assert_eq!(core::mem::size_of::<InitParams>(), 0);
    }

    #[test]
    fn new_from_init_params_sets_block_id_default() {
        let block = DeleteBlock::new_from_init_params(&InitParams);
        assert_eq!(block.block_id(), 0);
    }

    #[test]
    fn init_state_returns_default_state() {
        let block = DeleteBlock { block_id: 123 };
        let state = block.init_state();
        assert!(matches!(state, State));
    }

    #[test]
    fn execute_compiles_and_returns_defaults_when_no_output_state_intents_returned() {
        let block = DeleteBlock { block_id: 1 };
        let ctx = ExecutionContext { time: 0 }; // adjust if your ExecutionContext has more fields
        let state = State;

        // The #[execute] macro should adapt this "unit return" method to the full signature:
        // fn execute(&self, &ExecutionContext, Input, &State) -> (Output, State, Intents)
        let (out, state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_delete: false,
                },
                &state,
            )
            .unwrap();

        // We can't assert much about intents without knowing its concrete type,
        // but we can ensure the call works and defaults are returned.
        let _ = intents;
        assert!(matches!(out, Output));
        assert!(matches!(state_out, State));
    }

    #[test]
    fn execute_handles_should_delete_true() {
        let block = DeleteBlock { block_id: 1 };
        let ctx = ExecutionContext { time: 0 }; // adjust if needed
        let state = State;

        let (out, state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_delete: true,
                },
                &state,
            )
            .unwrap();

        let _ = intents;
        assert!(matches!(out, Output));
        assert!(matches!(state_out, State));
    }
}
