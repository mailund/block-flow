use ::block_macros::*;
use ::block_traits::intents::ZeroIntents;
use ::block_traits::ExecutionContextTrait;
use ::trade_types::{Cents, Contract, Price, Side};

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
    fn get_position(&self, _block_id: u32, _contract: &Contract) -> Option<trade_types::Quantity> {
        // mock position
        None
    }
}

mod self_only_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) {}
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_unit_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) {}
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_output {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Output {
            Output
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> State {
            State
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_intents {
    use block_traits::intents::ZeroIntents;

    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Intents {
            ZeroIntents::new()
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_output_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> (Output, State) {
            (Output, State)
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_output_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> (Output, Intents) {
            (Output, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_state_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> (State, Intents) {
            (State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_full_tuple {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> (Output, State, Intents) {
            (Output, State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_output_none {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<Output> {
            None
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_output_some {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<Output> {
            Some(Output)
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_output_state_none {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<(Output, State)> {
            None
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_output_state_some {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<(Output, State)> {
            Some((Output, State))
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_full_tuple_none {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<(Output, State, Intents)> {
            None
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod self_only_option_full_tuple_some {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self) -> Option<(Output, State, Intents)> {
            Some((Output, State, ZeroIntents::new()))
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

// FIXME: macro cannot handle Option<()> and () return types yet

// mod self_only_option_unit_none {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self) -> Option<()> {
//             None
//         }
//     }
// }

// mod self_only_option_unit_some {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self) -> Option<()> {
//             Some(())
//         }
//     }
// }

mod context_only_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) {
            let _ = context;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_unit_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) {
            let _ = context;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_output {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> Output {
            let _ = context;
            Output
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> State {
            let _ = context;
            State
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> Intents {
            let _ = context;
            ZeroIntents::new()
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_output_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> (Output, State) {
            let _ = context;
            (Output, State)
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_output_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> (Output, Intents) {
            let _ = context;
            (Output, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_state_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> (State, Intents) {
            let _ = context;
            (State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_only_full_tuple {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext) -> (Output, State, Intents) {
            let _ = context;
            (Output, State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

// mod context_only_option_unit_none {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, context: &ExecutionContext) -> Option<()> {
//             let _ = context;
//             None
//         }
//     }
// }

// mod context_only_option_unit_some {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, context: &ExecutionContext) -> Option<()> {
//             let _ = context;
//             Some(())
//         }
//     }
// }

mod input_only_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) {
            let _ = input;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_unit_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) {
            let _ = input;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_output {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> Output {
            let _ = input;
            Output
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> State {
            let _ = input;
            State
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> Intents {
            let _ = input;
            ZeroIntents::new()
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_output_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> (Output, State) {
            let _ = input;
            (Output, State)
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_output_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> (Output, Intents) {
            let _ = input;
            (Output, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_state_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> (State, Intents) {
            let _ = input;
            (State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_only_full_tuple {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input) -> (Output, State, Intents) {
            let _ = input;
            (Output, State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

// mod input_only_option_unit_none {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, input: Input) -> Option<()> {
//             let _ = input;
//             None
//         }
//     }
// }

// mod input_only_option_unit_some {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, input: Input) -> Option<()> {
//             let _ = input;
//             Some(())
//         }
//     }
// }

mod state_only_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) {
            let _ = state;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_unit_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) {
            let _ = state;
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_output {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> Output {
            let _ = state;
            Output
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> State {
            let _ = state;
            State
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> Intents {
            let _ = state;
            ZeroIntents::new()
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_output_state {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> (Output, State) {
            let _ = state;
            (Output, State)
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_output_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> (Output, Intents) {
            let _ = state;
            (Output, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_state_intents {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> (State, Intents) {
            let _ = state;
            (State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod state_only_full_tuple {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, state: &State) -> (Output, State, Intents) {
            let _ = state;
            (Output, State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

// mod state_only_option_unit_none {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, state: &State) -> Option<()> {
//             let _ = state;
//             None
//         }
//     }
// }

// mod state_only_option_unit_some {
//     use super::*;
//     make_defaults!(input, output, state, init_params);

//     #[block]
//     struct DummyBlock;
//     impl DummyBlock {
//         #[execute]
//         fn execute(&self, state: &State) -> Option<()> {
//             let _ = state;
//             Some(())
//         }
//     }
// }

mod context_input_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext, input: Input) {
            let _ = (context, input);
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_input_output {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext, input: Input) -> Output {
            let _ = (context, input);
            Output
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_state_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext, state: &State) {
            let _ = (context, state);
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod input_state_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, input: Input, state: &State) {
            let _ = (input, state);
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_input_state_no_return {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(&self, context: &ExecutionContext, input: Input, state: &State) {
            let _ = (context, input, state);
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}

mod context_input_state_full_tuple {
    use super::*;
    make_defaults!(input, output, state, init_params);

    #[block]
    #[allow(dead_code)]
    struct DummyBlock;
    impl DummyBlock {
        #[execute]
        fn execute(
            &self,
            context: &ExecutionContext,
            input: Input,
            state: &State,
        ) -> (Output, State, Intents) {
            let _ = (context, input, state);
            (Output, State, ZeroIntents::new())
        }
    }

    #[test]
    fn test_context_input_output_execution() {
        let block = DummyBlock;
        let context = ExecutionContext { time: 0 };
        let _ = block.execute(&context, Input, &State);
    }
}
