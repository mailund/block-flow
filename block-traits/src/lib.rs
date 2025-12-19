// Make ::block_traits work inside this crate (for proc-macro expansions).
// so we have the macros available for testing.
#[cfg(test)]
extern crate self as block_traits;

use channels::{Reader, Writer};

pub mod associated_types;
pub mod block_spec;
pub mod block_weave;
pub mod type_erasure;

// Re-export for convience
pub use execution_context::ExecutionContext;

pub use associated_types::{BlockInput, BlockOutput, BlockSpecAssociatedTypes};
pub use block_spec::BlockSpec;
pub use type_erasure::Block;

#[cfg(test)]
mod tests {
    use super::*;
    use channels::*;
    use std::cell::RefCell;

    // Test init parameter structs
    #[derive(serde::Serialize, serde::Deserialize)]
    struct DoublerInitParams;

    impl ::serialization::structs::Serializable for DoublerInitParams {}

    #[derive(serde::Serialize, serde::Deserialize)]
    struct AccumulatorInitParams;

    impl ::serialization::structs::Serializable for AccumulatorInitParams {}

    // Test implementations for unit tests
    #[derive(Clone, Debug, PartialEq)]
    struct TestInput {
        value: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestOutput {
        result: i32,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct TestInputKeys {
        value: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct TestOutputKeys;

    // Mock reader/writer for testing
    struct MockReader<T> {
        data: T,
    }

    impl<T: Clone> Reader<T> for MockReader<T> {
        fn read(&self) -> T {
            self.data.clone()
        }
    }

    struct MockWriter<T> {
        written: RefCell<Option<T>>,
    }

    impl<T: Clone> Writer<T> for MockWriter<T> {
        fn write(&self, data: &T) {
            *self.written.borrow_mut() = Some(data.clone());
        }
    }

    impl ::serialization::structs::Serializable for TestInputKeys {}
    impl ::serialization::structs::Serializable for TestOutputKeys {}

    impl ChannelKeys for TestInputKeys {
        fn channel_names(&self) -> Vec<String> {
            vec![self.value.clone()]
        }
    }

    impl InputKeys<TestInput> for TestInputKeys {
        type ReaderType = MockReader<TestInput>;

        fn reader(&self, _registry: &ChannelRegistry) -> Result<Self::ReaderType, RegistryError> {
            Ok(MockReader {
                data: TestInput { value: 0 },
            })
        }
    }

    impl ChannelKeys for TestOutputKeys {
        fn channel_names(&self) -> Vec<String> {
            vec![]
        }
    }

    impl OutputKeys<TestOutput> for TestOutputKeys {
        type WriterType = MockWriter<TestOutput>;

        fn writer(&self, _registry: &ChannelRegistry) -> Result<Self::WriterType, RegistryError> {
            Ok(MockWriter {
                written: RefCell::new(None),
            })
        }

        fn register(&self, _registry: &mut ChannelRegistry) {
            // Mock implementation
        }
    }

    impl BlockInput for TestInput {
        type Keys = TestInputKeys;
    }

    impl BlockOutput for TestOutput {
        type Keys = TestOutputKeys;
    }

    // Test block that doubles the input
    struct DoublerBlock;

    impl BlockSpecAssociatedTypes for DoublerBlock {
        type Input = TestInput;
        type Output = TestOutput;
        type State = i32; // Execution counter
        type InitParameters = DoublerInitParams;
        type Intents = ::intents::ZeroIntents;
    }

    impl BlockSpec for DoublerBlock {
        fn block_id(&self) -> u32 {
            8765
        }

        fn new_from_init_params(_params: &DoublerInitParams) -> Self {
            DoublerBlock
        }

        fn init_state(&self) -> Self::State {
            0
        }

        fn execute(
            &self,
            _context: &ExecutionContext,
            input: Self::Input,
            state: &Self::State,
        ) -> (Self::Output, Self::State, Self::Intents) {
            let output = TestOutput {
                result: input.value * 2,
            };
            (output, state + 1, Self::Intents::new())
        }
    }

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext { time: 12345 };
        assert_eq!(context.time, 12345);
    }

    #[test]
    fn test_block_spec_block_id() {
        let block = DoublerBlock;
        assert_eq!(block.block_id(), 8765);
    }

    #[test]
    fn test_block_spec_new_from_init_params() {
        let _block = DoublerBlock::new_from_init_params(&DoublerInitParams);
    }

    #[test]
    fn test_block_spec_init_state() {
        let block = DoublerBlock;
        let state = block.init_state();
        assert_eq!(state, 0);
    }

    #[test]
    fn test_block_spec_execute() {
        let block = DoublerBlock;
        let context = ExecutionContext { time: 100 };
        let input = TestInput { value: 21 };
        let state = 0;

        let (output, new_state, _intents) = block.execute(&context, input, &state);

        assert_eq!(output.result, 42);
        assert_eq!(new_state, 1);
    }

    #[test]
    fn test_block_spec_execute_multiple_times() {
        let block = DoublerBlock;
        let context = ExecutionContext { time: 100 };
        let input = TestInput { value: 5 };
        let mut state = block.init_state();

        for expected_count in 1..=3 {
            let (output, new_state, _intents) = block.execute(&context, input.clone(), &state);
            assert_eq!(output.result, 10); // 5 * 2
            assert_eq!(new_state, expected_count);
            state = new_state;
        }
    }

    #[test]
    fn test_channel_keys_and_reader_writer_traits_are_invoked() {
        // This test explicitly invokes the trait-required methods that are easy to miss in coverage.
        let keys_in = TestInputKeys {
            value: "input_channel".to_string(),
        };
        let keys_out = TestOutputKeys;

        // Prefer Default if the registry supports it, otherwise use new().
        let mut registry: ChannelRegistry = Default::default();

        // ChannelKeys::channel_names
        let names = keys_in.channel_names();
        assert_eq!(names, vec!["input_channel".to_string()]);
        let out_names = keys_out.channel_names();
        assert!(out_names.is_empty());

        // OutputKeys::register (no-op mock, but we still invoke it)
        keys_out.register(&mut registry);

        // InputKeys::reader + Reader::read
        let reader = keys_in.reader(&registry).unwrap();
        let read_value = reader.read();
        assert_eq!(read_value, TestInput { value: 0 });

        // OutputKeys::writer + Writer::write
        let writer = keys_out.writer(&registry).unwrap();
        writer.write(&TestOutput { result: 123 });
        assert_eq!(writer.written.borrow().as_ref().unwrap().result, 123);
    }

    #[test]
    fn test_wrapped_block_new() {
        let block = DoublerBlock;
        let reader = MockReader {
            data: TestInput { value: 7 },
        };
        let writer = MockWriter {
            written: RefCell::new(None),
        };

        let wrapped = type_erasure::EncapsulatedBlock::new(block, reader, writer);
        assert_eq!(*wrapped.state_cell.borrow(), 0); // Should be initialized
    }

    #[test]
    fn test_wrapped_block_execute() {
        let block = DoublerBlock;
        let reader = MockReader {
            data: TestInput { value: 15 },
        };
        let writer = MockWriter {
            written: RefCell::new(None),
        };

        let wrapped = type_erasure::EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 200 };

        use type_erasure::TypeErasedBlock; // Get trait in scope for execute.
        wrapped.execute(&context);

        let written_data = wrapped.output_writer.written.borrow();
        assert!(written_data.is_some());
        assert_eq!(written_data.as_ref().unwrap().result, 30); // 15 * 2

        assert_eq!(*wrapped.state_cell.borrow(), 1);
    }

    #[test]
    fn test_multiple_wrapped_block_executions() {
        let block = DoublerBlock;
        let reader = MockReader {
            data: TestInput { value: 3 },
        };
        let writer = MockWriter {
            written: RefCell::new(None),
        };

        use type_erasure::TypeErasedBlock; // Get trait in scope for execute.
        let wrapped = type_erasure::EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 300 };

        for expected_state in 1..=5 {
            wrapped.execute(&context);
            assert_eq!(*wrapped.state_cell.borrow(), expected_state);

            let written_data = wrapped.output_writer.written.borrow();
            assert_eq!(written_data.as_ref().unwrap().result, 6); // 3 * 2
        }
    }

    // Test block with more complex state
    struct AccumulatorBlock;

    impl BlockSpecAssociatedTypes for AccumulatorBlock {
        type Input = TestInput;
        type Output = TestOutput;
        type State = i32; // Accumulates input values
        type InitParameters = AccumulatorInitParams;
        type Intents = ::intents::ZeroIntents;
    }

    impl BlockSpec for AccumulatorBlock {
        fn block_id(&self) -> u32 {
            42
        }

        fn new_from_init_params(_params: &AccumulatorInitParams) -> Self {
            AccumulatorBlock
        }

        fn init_state(&self) -> Self::State {
            0
        }

        fn execute(
            &self,
            _context: &ExecutionContext,
            input: Self::Input,
            state: &Self::State,
        ) -> (Self::Output, Self::State, Self::Intents) {
            let new_state = state + input.value;
            let output = TestOutput { result: new_state };
            (output, new_state, Self::Intents::new())
        }
    }

    #[test]
    fn test_accumulator_block_block_id_and_new_from_init_params() {
        let block = AccumulatorBlock;
        assert_eq!(block.block_id(), 42);

        let _block2 = AccumulatorBlock::new_from_init_params(&AccumulatorInitParams);
    }

    #[test]
    fn test_accumulator_block() {
        let block = AccumulatorBlock;
        let context = ExecutionContext { time: 400 };
        let mut state = block.init_state();

        let inputs = vec![
            TestInput { value: 5 },
            TestInput { value: 10 },
            TestInput { value: 3 },
        ];

        let expected_results = vec![5, 15, 18];

        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let (output, new_state, _intents) = block.execute(&context, input, &state);
            assert_eq!(output.result, expected);
            assert_eq!(new_state, expected);
            state = new_state;
        }
    }
}
