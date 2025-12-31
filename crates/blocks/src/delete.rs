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
    fn execute<E: EffectConsumerTrait>(
        &self,
        Input { should_delete }: Input,
        effects: &mut E,
    ) -> Result<(), execute_status::FailureStatus> {
        if should_delete {
            effects.schedule_terminate_effect()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_traits::Effect;
    use trade_types::{Cents, Contract, Price, Side};

    pub struct OrderBook;

    impl block_traits::execution_context::OrderBookTrait for OrderBook {
        fn top_of_side(&self, _side: Side) -> Option<Price> {
            // Dummy implementation
            Some(Price::from(Cents(100)))
        }
    }

    pub struct ExecutionContext {
        pub time: u64,
    }

    impl ExecutionContextTrait for ExecutionContext {
        type OrderBook = OrderBook;

        fn time(&self) -> u64 {
            self.time
        }
        fn get_order_book(&self, _contract: &Contract) -> Option<OrderBook> {
            // Mock implementation
            Some(OrderBook {})
        }
        fn get_position(
            &self,
            _block_id: u32,
            _contract: &Contract,
        ) -> Option<trade_types::Quantity> {
            // mock position
            None
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
        let mut effect_handler = |_effect: Effect| {};

        // The #[execute] macro should adapt this "unit return" method to the full signature:
        // fn execute(&self, &ExecutionContext, Input, &State) -> (Output, State, Intents)
        let (out, state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_delete: false,
                },
                &state,
                &mut effect_handler,
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
        let mut effect_handler = |_effect: Effect| {};

        let (out, state_out, intents) = block
            .execute(
                &ctx,
                Input {
                    should_delete: true,
                },
                &state,
                &mut effect_handler,
            )
            .unwrap();

        let _ = intents;
        assert!(matches!(out, Output));
        assert!(matches!(state_out, State));
    }
}
