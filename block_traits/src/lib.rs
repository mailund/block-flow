use channels::{Reader, Writer};
use intents::{BlockIntents, Intent};
use std::ops::Deref;

pub mod serialization;

/// Trait for block input data types.
///
/// This trait defines the associated types needed for a block's input.
/// The `Keys` type must implement `InputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use ::channels::*;
/// use ::serde::{Serialize, Deserialize};
/// use ::serialization::structs::Serializable;
///
/// // Define a simple input type
/// #[derive(Clone)]
/// struct SimpleInput {
///     value: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize, Debug, Clone)]
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
/// use ::block_traits::*;
/// use ::channels::*;
/// use ::serde::{Serialize, Deserialize};
/// use ::serialization::structs::Serializable;
///
/// // Define a simple output type
/// #[derive(Clone)]
/// struct SimpleOutput {
///     result: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub state_cell: std::cell::RefCell<B::State>, // RefCell to allow interior mutability
}

impl<B: BlockSpec> EncapsulatedBlock<B> {
    pub fn new(
        block: B,
        input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
        output_writer:
            <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    ) -> Self {
        let init_state = block.init_state();
        let state_cell = std::cell::RefCell::new(init_state);
        Self {
            block,
            input_reader,
            output_writer,
            state_cell,
        }
    }
}

pub trait TypeErasedBlock {
    fn execute(&self, context: &ExecutionContext) -> Vec<Intent>;
}

impl<B: BlockSpec> TypeErasedBlock for EncapsulatedBlock<B> {
    fn execute(&self, context: &ExecutionContext) -> Vec<Intent> {
        // Get the input for the execution from channels and the stored state.
        let input = self.input_reader.read();
        let old_state = self.state_cell.borrow();

        // Execute the block logic.
        let (output, new_state, intents) = self.block.execute(context, input, &old_state);

        // Write values to channels and state
        drop(old_state); // Explicitly drop borrow before mutable borrow
        self.output_writer.write(&output);
        *self.state_cell.borrow_mut() = new_state;

        // Return the intents as a vector; this is also a type-erasure point
        // since we now no longer care about the specific intent count.
        intents.as_slice().to_vec()
    }
}

/// Type-erased block for execution in a weaved
/// execution plan.
pub struct Block {
    block: Box<dyn TypeErasedBlock>,
}
impl Block {
    pub fn new(block: Box<dyn TypeErasedBlock>) -> Self {
        Self { block }
    }

    pub fn execute(&self, context: &ExecutionContext) -> Vec<Intent> {
        self.block.execute(context).as_slice().to_vec()
    }
}

/// Topologically ordered blocks for execution in a weave.
pub struct TopoOrderedBlocks(pub Vec<Block>);

impl Deref for TopoOrderedBlocks {
    type Target = Vec<Block>;
    fn deref(&self) -> &Self::Target {
        &self.0
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

        let wrapped = EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 200 };

        wrapped.execute(&context);

        // Check that output was written
        let written_data = wrapped.output_writer.written.borrow();
        assert!(written_data.is_some());
        assert_eq!(written_data.as_ref().unwrap().result, 30); // 15 * 2

        // Check that state was updated
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

        let wrapped = EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 300 };

        // Execute multiple times
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

#[cfg(test)]
extern crate self as block_traits;

#[cfg(test)]
mod example_block_tests {
    use super::serialization::BlockNode;
    use super::*;
    use adder_block::*;
    use block_macros::{block, init_params, input, output, state};
    use channels::{ChannelRegistry, InputKeys, OutputKeys};

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
            pub block_id: u32,
            pub offset: i32,
        }

        impl BlockSpec for AdderBlock {
            fn block_id(&self) -> u32 {
                self.block_id
            }

            fn new_from_init_params(params: &InitParams) -> Self {
                Self {
                    block_id: 0,
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
            ) -> (Output, State, Self::Intents) {
                let result = input.a + input.b + self.offset;
                let new_state = State {
                    call_count: state.call_count + 1,
                };

                let output = Output { sum: result };

                (output, new_state, ::intents::ZeroIntents::new())
            }
        }
    }

    #[test]
    fn test_adder_block_pure_execution() {
        let block = AdderBlock::new_from_init_params(&InitParams { offset: 10 });

        let input = Input { a: 5, b: 3 };
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output, new_state, _intents) = block.execute(&context, input, &state);

        assert_eq!(output.sum, 18); // 5 + 3 + 10
        assert_eq!(new_state.call_count, 1);
    }

    #[test]
    fn test_adder_block_multiple_calls() {
        let block = AdderBlock::new_from_init_params(&InitParams { offset: 0 });
        let state = block.init_state();
        let context = ExecutionContext { time: 0 };

        let (output1, new_state1, _intents) = block.execute(&context, Input { a: 1, b: 2 }, &state);
        assert_eq!(output1.sum, 3);
        assert_eq!(new_state1.call_count, 1);

        let (output2, new_state2, _intents) =
            block.execute(&context, Input { a: 10, b: 20 }, &new_state1);
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

        let wired = super::serialization::BlockSerialisation::new_node::<AdderBlock>(
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

        let wired = super::serialization::BlockSerialisation::new_node::<AdderBlock>(
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

    #[test]
    fn get_node_from_json() {
        let json = r#"
        {
            "input_keys": {
                "a": "input_a",
                "b": "input_b"
            },
            "output_keys": {
                "sum": "output_sum"
            },
            "init_params": {
                "offset": 5
            }
        }
        "#;

        let summary: super::serialization::BlockSerializationSummary<adder_block::AdderBlock> =
            ::serialization::read_struct_from_json(json.as_bytes()).unwrap();

        assert_eq!(summary.init_params.offset, 5);
        assert_eq!(summary.input_keys.a, "input_a");
        assert_eq!(summary.input_keys.b, "input_b");
        assert_eq!(summary.output_keys.sum, "output_sum");
    }
}
