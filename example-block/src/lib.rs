use block_macros::{block, init_params, input, output, state};
use block_traits::{BlockSpec, ExecutionContext};

pub mod adder_block {
    use super::*;

    /// A simple adder block that adds two numbers
    #[input]
    pub struct Input {
        pub a: i32,
        pub b: i32,
    }

    /// Output for the adder block
    #[output]
    pub struct Output {
        pub sum: i32,
    }

    /// State for the adder block
    #[state]
    pub struct State {
        pub call_count: u32,
    }

    #[init_params]
    pub struct InitParams {
        pub offset: i32,
    }

    #[block]
    pub struct AdderBlock {
        pub offset: i32,
    }

    impl BlockSpec for AdderBlock {
        fn new_from_init_params(params: &InitParams) -> Self {
            Self {
                offset: params.offset,
            }
        }

        /// Initialize state for this block
        fn init_state(&self) -> State {
            State { call_count: 0 }
        }

        /// Execute the block logic
        fn execute(
            &self,
            _context: &ExecutionContext,
            input: Input,
            state: &State,
        ) -> (Output, State) {
            let result = input.a + input.b + self.offset;
            let new_state = State {
                call_count: state.call_count + 1,
            };

            let output = Output { sum: result };
            (output, new_state)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::adder_block::*;
    use super::*;
    use block_traits::{BlockInput, BlockOutput};
    use channels::{ChannelRegistry, InputKeys, OutputKeys};
    use weave::BlockNode;

    #[test]
    fn test_adder_block_pure_execution() {
        let block = AdderBlock::new_from_init_params(&InitParams { offset: 10 });

        let input = Input { a: 5, b: 3 };
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output, new_state) = block.execute(&context, input, &state);

        assert_eq!(output.sum, 18); // 5 + 3 + 10
        assert_eq!(new_state.call_count, 1);
    }

    #[test]
    fn test_adder_block_multiple_calls() {
        let block = AdderBlock::new_from_init_params(&InitParams { offset: 0 });
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output1, new_state1) = block.execute(&context, Input { a: 1, b: 2 }, &state);
        assert_eq!(output1.sum, 3);
        assert_eq!(new_state1.call_count, 1);

        let (output2, new_state2) = block.execute(&context, Input { a: 10, b: 20 }, &new_state1);
        assert_eq!(output2.sum, 30);
        assert_eq!(new_state2.call_count, 2);
    }

    #[test]
    fn test_input_output_readers() {
        let mut registry = ChannelRegistry::new();
        registry.put("input_a", 7);
        registry.put("input_b", 13);
        registry.put("output_sum", 0);

        type InputKeys = <Input as BlockInput>::Keys;
        type OutputKeys = <Output as BlockOutput>::Keys;

        let in_keys = InputKeys {
            a: "input_a".to_string(),
            b: "input_b".to_string(),
        };

        let out_keys = OutputKeys {
            sum: "output_sum".to_string(),
        };

        // Test InputKeys trait
        let reader = in_keys.reader(&registry).unwrap();
        let input = reader.read();
        assert_eq!(input.a, 7);
        assert_eq!(input.b, 13);

        // Test OutputKeys trait
        let writer = out_keys.writer(&registry).unwrap();
        let output = Output { sum: 42 };
        writer.write(&output);

        let result = registry.get::<i32>("output_sum").unwrap();
        assert_eq!(*result.borrow(), 42);
    }

    #[test]
    fn test_adder_block_with_registry() {
        let mut registry = ChannelRegistry::new();

        // Setup input data
        registry.put("input_a", 7);
        registry.put("input_b", 13);

        type InputKeys = <Input as BlockInput>::Keys;
        type OutputKeys = <Output as BlockOutput>::Keys;

        let in_keys = InputKeys {
            a: "input_a".to_string(),
            b: "input_b".to_string(),
        };

        let out_keys = OutputKeys {
            sum: "output_sum".to_string(),
        };

        let mut wired = weave::BlockSerialisation::new_node::<AdderBlock>(
            in_keys.clone(),
            out_keys.clone(),
            InitParams { offset: 100 },
        )
        .weave(&mut registry)
        .unwrap();

        // Execute one tick
        let context = ExecutionContext { time: 0 };
        wired.execute(&context);

        // Check output in registry
        let result = registry.get::<i32>("output_sum").unwrap();
        assert_eq!(*result.borrow(), 120); // 7 + 13 + 100
    }

    #[test]
    fn test_adder_block_registry_updates() {
        let mut registry = ChannelRegistry::new();

        // Setup input data
        registry.put("a", 1);
        registry.put("b", 2);

        type InputKeys = <Input as BlockInput>::Keys;
        type OutputKeys = <Output as BlockOutput>::Keys;

        let in_keys = InputKeys {
            a: "a".to_string(),
            b: "b".to_string(),
        };

        let out_keys = OutputKeys {
            sum: "sum".to_string(),
        };

        let mut wired = weave::BlockSerialisation::new_node::<AdderBlock>(
            in_keys.clone(),
            out_keys.clone(),
            InitParams { offset: 0 },
        )
        .weave(&mut registry)
        .unwrap();

        // First tick
        let context = ExecutionContext { time: 0 };
        wired.execute(&context);
        let result = registry.get::<i32>("sum").unwrap();
        assert_eq!(*result.borrow(), 3);

        // Update inputs in registry
        let a_ref = registry.get::<i32>("a").unwrap();
        let b_ref = registry.get::<i32>("b").unwrap();
        *a_ref.borrow_mut() = 10;
        *b_ref.borrow_mut() = 20;

        // Second tick should see updated values
        wired.execute(&context);
        let result = registry.get::<i32>("sum").unwrap();
        assert_eq!(*result.borrow(), 30);
    }

    #[test]
    fn test_input_macro() {
        use block_macros::input;

        // Define a test input struct using the macro
        #[input]
        struct TestInput {
            x: i32,
            y: f64,
        }

        let mut registry = ChannelRegistry::new();
        registry.put("x_val", 42i32);
        registry.put("y_val", 3.5f64);

        let keys = TestInputKeys {
            x: "x_val".to_string(),
            y: "y_val".to_string(),
        };

        // Create reader using the trait method
        let reader = keys.reader(&registry).unwrap();
        let input = reader.read();

        assert_eq!(input.x, 42);
        assert_eq!(input.y, 3.5);
    }
}
