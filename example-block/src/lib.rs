use block_macros::{block, input, output, state};
use blocks::{BlockSpec, ExecutionContext, WrappedBlock};
use registry::{InputKeys, OutputKeys, Registry, RegistryError};

pub mod adder_block {
    use super::*;

    /// A simple adder block that adds two numbers
    #[input]
    pub struct AdderInput {
        pub a: i32,
        pub b: i32,
    }

    /// Output for the adder block
    #[output]
    pub struct AdderOutput {
        pub sum: i32,
    }

    /// State for the adder block
    #[state]
    pub struct AdderState {
        pub call_count: u32,
    }

    #[block(input = AdderInput, output = AdderOutput, state = AdderState)]
    pub struct AdderBlock {
        pub offset: i32,
    }

    impl AdderBlock {
        /// Constructor
        pub fn new(offset: i32) -> Self {
            Self { offset }
        }
    }

    impl BlockSpec for AdderBlock {
        /// Initialize state for this block
        fn init_state(&self) -> AdderState {
            AdderState { call_count: 0 }
        }

        /// Execute the block logic
        fn execute(
            &self,
            _context: &ExecutionContext,
            input: AdderInput,
            state: &AdderState,
        ) -> (AdderOutput, AdderState) {
            let result = input.a + input.b + self.offset;
            let new_state = AdderState {
                call_count: state.call_count + 1,
            };

            let output = AdderOutput { sum: result };
            (output, new_state)
        }
    }

    impl AdderBlock {
        // === THIS WILL BE FIXED LATER ==========

        /// Wire the block to the registry
        pub fn wire(
            &self,
            registry: &Registry,
            in_keys: &<AdderInput as blocks::BlockInput>::Keys,
            out_keys: &<AdderOutput as blocks::BlockOutput>::Keys,
        ) -> Result<AdderWiredBlock, RegistryError> {
            use super::{InputKeys, OutputKeys};

            // Create readers/writers that capture the Rc references
            let input_reader = in_keys.reader(registry)?;
            let output_writer = out_keys.writer(registry)?;

            let state = self.init_state();

            Ok(AdderWiredBlock {
                block: AdderBlock::new(self.offset),
                input_reader,
                output_writer,
                state,
            })
        }

        /// Declare and wire in one step
        pub fn declare_and_wire(
            &self,
            registry: &mut Registry,
            in_keys: &<AdderInput as blocks::BlockInput>::Keys,
            out_keys: &<AdderOutput as blocks::BlockOutput>::Keys,
        ) -> Result<AdderWiredBlock, RegistryError> {
            self.register_outputs(registry, out_keys);
            self.wire(registry, in_keys, out_keys)
        }
    }
}

/// A wired block that can be ticked
type AdderWiredBlock = WrappedBlock<adder_block::AdderBlock>;

#[cfg(test)]
mod tests {
    use super::adder_block::*;
    use super::*;
    use blocks::{Block, BlockInput, BlockOutput};

    #[test]
    fn test_adder_block_pure_execution() {
        let block = AdderBlock::new(10);

        let input = AdderInput { a: 5, b: 3 };
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output, new_state) = block.execute(&context, input, &state);

        assert_eq!(output.sum, 18); // 5 + 3 + 10
        assert_eq!(new_state.call_count, 1);
    }

    #[test]
    fn test_adder_block_multiple_calls() {
        let block = AdderBlock::new(0);
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output1, new_state1) = block.execute(&context, AdderInput { a: 1, b: 2 }, &state);
        assert_eq!(output1.sum, 3);
        assert_eq!(new_state1.call_count, 1);

        let (output2, new_state2) =
            block.execute(&context, AdderInput { a: 10, b: 20 }, &new_state1);
        assert_eq!(output2.sum, 30);
        assert_eq!(new_state2.call_count, 2);
    }

    #[test]
    fn test_input_output_readers() {
        let mut registry = Registry::new();
        registry.put("input_a", 7);
        registry.put("input_b", 13);
        registry.put("output_sum", 0);

        type InputKeys = <AdderInput as BlockInput>::Keys;
        type OutputKeys = <AdderOutput as BlockOutput>::Keys;

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
        let output = AdderOutput { sum: 42 };
        writer.write(&output);

        let result = registry.get::<i32>("output_sum").unwrap();
        assert_eq!(*result.borrow(), 42);
    }

    #[test]
    fn test_adder_block_with_registry() {
        let mut registry = Registry::new();
        let block = AdderBlock::new(100);

        // Setup input data
        registry.put("input_a", 7);
        registry.put("input_b", 13);

        type InputKeys = <AdderInput as BlockInput>::Keys;
        type OutputKeys = <AdderOutput as BlockOutput>::Keys;

        let in_keys = InputKeys {
            a: "input_a".to_string(),
            b: "input_b".to_string(),
        };

        let out_keys = OutputKeys {
            sum: "output_sum".to_string(),
        };

        // Declare and wire the block
        let mut wired = block
            .declare_and_wire(&mut registry, &in_keys, &out_keys)
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
        let mut registry = Registry::new();
        let block = AdderBlock::new(0);

        // Setup input data
        registry.put("a", 1);
        registry.put("b", 2);

        type InputKeys = <AdderInput as BlockInput>::Keys;
        type OutputKeys = <AdderOutput as BlockOutput>::Keys;

        let in_keys = InputKeys {
            a: "a".to_string(),
            b: "b".to_string(),
        };

        let out_keys = OutputKeys {
            sum: "sum".to_string(),
        };

        let mut wired = block
            .declare_and_wire(&mut registry, &in_keys, &out_keys)
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

        let mut registry = Registry::new();
        registry.put("x_val", 42i32);
        registry.put("y_val", 3.14f64);

        let keys = TestInputKeys {
            x: "x_val".to_string(),
            y: "y_val".to_string(),
        };

        // Create reader using the trait method
        let reader = keys.reader(&registry).unwrap();
        let input = reader.read();

        assert_eq!(input.x, 42);
        assert_eq!(input.y, 3.14);
    }
}
