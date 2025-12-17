/// Trait for block input data types.
///
/// This trait defines the associated types needed for a block's input.
/// The `Keys` type must implement `InputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use channels::*;
/// use serde::{Serialize, Deserialize};
/// use serialization::structs::Serializable;
///
/// // Define a simple input type
/// #[derive(Clone)]
/// struct SimpleInput {
///     value: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize)]
/// struct SimpleInputKeys;
///
/// impl Serializable for SimpleInputKeys {}
///
/// impl ChannelKeys for SimpleInputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// // Mock reader for testing
/// struct MockReader<T> { data: T }
/// impl<T: Clone> Reader<T> for MockReader<T> {
///     fn read(&self) -> T { self.data.clone() }
/// }
///
/// impl InputKeys<SimpleInput> for SimpleInputKeys {
///     type ReaderType = MockReader<SimpleInput>;
///
///     fn reader(&self, _registry: &ChannelRegistry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: SimpleInput { value: 42 } })
///     }
/// }
///
/// impl BlockInput for SimpleInput {
///     type Keys = SimpleInputKeys;
/// }
/// ```
pub trait BlockInput: Sized {
    type Keys: ::channels::InputKeys<Self> + ::serialization::structs::Serializable;
}

/// Trait for block output data types.
///
/// This trait defines the associated types needed for a block's output.
/// The `Keys` type must implement `OutputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use channels::*;
/// use serde::{Serialize, Deserialize};
/// use serialization::structs::Serializable;
///
/// // Define a simple output type
/// #[derive(Clone)]
/// struct SimpleOutput {
///     result: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize)]
/// struct SimpleOutputKeys;
///
/// impl Serializable for SimpleOutputKeys {}
///
/// impl ChannelKeys for SimpleOutputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// // Mock writer for testing
/// struct MockWriter<T> {
///     written: std::cell::RefCell<Option<T>>
/// }
///
/// impl<T: Clone> Writer<T> for MockWriter<T> {
///     fn write(&self, data: &T) {
///         *self.written.borrow_mut() = Some(data.clone());
///     }
/// }
///
/// impl OutputKeys<SimpleOutput> for SimpleOutputKeys {
///     type WriterType = MockWriter<SimpleOutput>;
///
///     fn writer(&self, _registry: &ChannelRegistry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///
///     fn register(&self, _registry: &mut ChannelRegistry) {
///         // Registration logic would go here
///     }
/// }
///
/// impl BlockOutput for SimpleOutput {
///     type Keys = SimpleOutputKeys;
/// }
/// ```
pub trait BlockOutput: Sized {
    type Keys: ::channels::OutputKeys<Self> + ::serialization::structs::Serializable;
}

/// Associated types for block specifications.
///
/// This trait groups the three core types that define a block:
/// - `Input`: The data the block consumes
/// - `Output`: The data the block produces
/// - `State`: The internal state the block maintains between executions
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use channels::*;
/// use serde::{Serialize, Deserialize};
/// use serialization::structs::Serializable;
///
/// #[derive(Clone)]
/// struct NoInput;
///
/// #[derive(Clone)]
/// struct CountOutput { count: i32 }
///
/// #[derive(Serialize, Deserialize)]
/// struct NoInputKeys;
///
/// #[derive(Serialize, Deserialize)]
/// struct CountOutputKeys;
///
/// #[derive(Serialize, Deserialize)]
/// struct CounterInitParams {
///     initial_count: i32,
/// }
///
/// impl Serializable for NoInputKeys {}
/// impl Serializable for CountOutputKeys {}
/// impl Serializable for CounterInitParams {}
///
/// impl ChannelKeys for NoInputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// impl ChannelKeys for CountOutputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// // Mock reader/writer
/// struct MockReader<T> { data: T }
/// impl<T: Clone> Reader<T> for MockReader<T> {
///     fn read(&self) -> T { self.data.clone() }
/// }
///
/// struct MockWriter<T> { written: std::cell::RefCell<Option<T>> }
/// impl<T: Clone> Writer<T> for MockWriter<T> {
///     fn write(&self, data: &T) { *self.written.borrow_mut() = Some(data.clone()); }
/// }
///
/// impl InputKeys<NoInput> for NoInputKeys {
///     type ReaderType = MockReader<NoInput>;
///     fn reader(&self, _registry: &ChannelRegistry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: NoInput })
///     }
/// }
///
/// impl OutputKeys<CountOutput> for CountOutputKeys {
///     type WriterType = MockWriter<CountOutput>;
///     fn writer(&self, _registry: &ChannelRegistry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///     fn register(&self, _registry: &mut ChannelRegistry) {}
/// }
///
/// impl BlockInput for NoInput {
///     type Keys = NoInputKeys;
/// }
///
/// impl BlockOutput for CountOutput {
///     type Keys = CountOutputKeys;
/// }
///
/// // Define associated types for a counter block
/// struct CounterBlockSpec;
///
/// impl BlockSpecAssociatedTypes for CounterBlockSpec {
///     type Input = NoInput;                 // No input needed
///     type Output = CountOutput;            // Outputs a count
///     type State = i32;                     // Internal counter state
///     type InitParameters = CounterInitParams;
///     type Intents = ::intents::ZeroIntents;
/// }
/// ```
pub trait BlockSpecAssociatedTypes {
    type Input: BlockInput;
    type Output: BlockOutput;
    type State; // FIXME: Should be serializable at some point
    type InitParameters: ::serialization::structs::Serializable;
    type Intents: ::intents::BlockIntents;
}

/// Execution context passed to blocks during execution.
pub struct ExecutionContext {
    pub time: u64,
}

/// Main trait for defining block behavior.
///
/// This trait extends `BlockSpecAssociatedTypes` with the core execution logic.
/// Blocks must implement `init_state` and `execute`, while the registry integration
/// methods have default implementations.
///
/// # Examples
///
/// ```rust
/// use block_macros::{block, init_params, input, output, state};
/// use block_traits::{BlockSpec, ExecutionContext};
/// use intents::ZeroIntents;
///
/// #[input]
/// pub struct Input;
///
/// #[output]
/// pub struct Output {
///     pub is_after: bool,
/// }
///
/// #[state]
/// pub struct State;
///
/// #[init_params]
/// pub struct InitParams {
///     pub time: u64,
/// }
///
/// #[block]
/// pub struct AfterBlock {
///     pub block_id: u32,
///     time: u64,
/// }
///
/// impl BlockSpec for AfterBlock {
///     fn block_id(&self) -> u32 {
///         self.block_id
///     }
///
///     fn new_from_init_params(params: &InitParams) -> Self {
///         AfterBlock {
///             block_id: 0,
///             time: params.time,
///         }
///     }
///
///     fn init_state(&self) -> State {
///         State
///     }
///
///     fn execute(
///         &self,
///         context: &ExecutionContext,
///         _input: Input,
///         _state: &State,
///     ) -> (Output, State, Self::Intents) {
///         let is_after = context.time > self.time;
///         let output = Output { is_after };
///         (output, State, ZeroIntents::new())
///     }
/// }

/// ```
pub trait BlockSpec: BlockSpecAssociatedTypes {
    fn block_id(&self) -> u32;

    fn init_state(&self) -> Self::State;

    fn new_from_init_params(params: &Self::InitParameters) -> Self;

    fn execute(
        &self,
        context: &ExecutionContext,
        input: Self::Input,
        state: &Self::State,
    ) -> (Self::Output, Self::State, Self::Intents);
}

pub struct EncapsulatedBlock<B: BlockSpec> {
    pub block: B,
    pub input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
    pub output_writer:
        <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    pub state: B::State,
}

impl<B: BlockSpec> EncapsulatedBlock<B> {
    pub fn new(
        block: B,
        input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
        output_writer:
            <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    ) -> Self {
        let state = block.init_state();
        Self {
            block,
            input_reader,
            output_writer,
            state,
        }
    }
}

pub trait TypeErasedBlock {
    fn execute(&mut self, context: &ExecutionContext);
}

impl<B: BlockSpec> TypeErasedBlock for EncapsulatedBlock<B> {
    fn execute(&mut self, context: &ExecutionContext) {
        use channels::{Reader, Writer};
        let input = self.input_reader.read();
        let (output, new_state, _intents) = self.block.execute(context, input, &self.state);
        // FIXME: ignoring intents for now
        self.output_writer.write(&output);
        self.state = new_state;
    }
}

pub struct Block {
    block: Box<dyn TypeErasedBlock>,
}
impl Block {
    pub fn new(block: Box<dyn TypeErasedBlock>) -> Self {
        Self { block }
    }

    pub fn execute(&mut self, context: &ExecutionContext) {
        self.block.execute(context);
    }
}

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

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestInputKeys {
        value: String,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
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

        // Execute multiple times to test state changes
        for expected_count in 1..=3 {
            let (output, new_state, _intents) = block.execute(&context, input.clone(), &state);
            assert_eq!(output.result, 10); // 5 * 2
            assert_eq!(new_state, expected_count);
            state = new_state;
        }
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

        let wrapped = EncapsulatedBlock::new(block, reader, writer);
        assert_eq!(wrapped.state, 0); // Should be initialized
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

        let mut wrapped = EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 200 };

        wrapped.execute(&context);

        // Check that output was written
        let written_data = wrapped.output_writer.written.borrow();
        assert!(written_data.is_some());
        assert_eq!(written_data.as_ref().unwrap().result, 30); // 15 * 2

        // Check that state was updated
        assert_eq!(wrapped.state, 1);
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

        let mut wrapped = EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 300 };

        // Execute multiple times
        for expected_state in 1..=5 {
            wrapped.execute(&context);
            assert_eq!(wrapped.state, expected_state);

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
    fn test_accumulator_block() {
        let block = AccumulatorBlock;
        let context = ExecutionContext { time: 400 };
        let mut state = block.init_state();

        // Accumulate values: 5, 10, 3
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
