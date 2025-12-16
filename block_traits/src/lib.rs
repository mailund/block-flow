/// Trait for block input data types.
///
/// This trait defines the associated types needed for a block's input.
/// The `Keys` type must implement `InputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use registry::*;
///
/// // Define a simple input type
/// #[derive(Clone)]
/// struct SimpleInput {
///     value: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// struct SimpleInputKeys;
///
/// impl InputKeys<SimpleInput> for SimpleInputKeys {
///     type ReaderType = MockReader<SimpleInput>;
///     
///     fn reader(&self, _registry: &Registry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: SimpleInput { value: 42 } })
///     }
/// }
///
/// impl BlockInput for SimpleInput {
///     type Keys = SimpleInputKeys;
/// }
///
/// // Mock reader for testing
/// struct MockReader<T> { data: T }
/// impl<T: Clone> Reader<T> for MockReader<T> {
///     fn read(&self) -> T { self.data.clone() }
/// }
/// ```
pub trait BlockInput: Sized {
    type Keys: registry::InputKeys<Self>;
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
/// use registry::*;
///
/// // Define a simple output type
/// #[derive(Clone)]
/// struct SimpleOutput {
///     result: f64,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// struct SimpleOutputKeys;
///
/// impl OutputKeys<SimpleOutput> for SimpleOutputKeys {
///     type WriterType = MockWriter<SimpleOutput>;
///     
///     fn writer(&self, _registry: &Registry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///     
///     fn register(&self, _registry: &mut Registry) {
///         // Registration logic would go here
///     }
/// }
///
/// impl BlockOutput for SimpleOutput {
///     type Keys = SimpleOutputKeys;
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
/// ```
pub trait BlockOutput: Sized {
    type Keys: registry::OutputKeys<Self>;
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
/// use registry::*;
///
/// // Define wrapper types that implement the required traits
/// #[derive(Clone)]
/// struct NoInput;
///
/// #[derive(Clone)]
/// struct CountOutput { count: i32 }
///
/// // Mock implementations
/// struct NoInputKeys;
/// struct CountOutputKeys;
///
/// impl InputKeys<NoInput> for NoInputKeys {
///     type ReaderType = MockReader<NoInput>;
///     fn reader(&self, _registry: &Registry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: NoInput })
///     }
/// }
///
/// impl OutputKeys<CountOutput> for CountOutputKeys {
///     type WriterType = MockWriter<CountOutput>;
///     fn writer(&self, _registry: &Registry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///     fn register(&self, _registry: &mut Registry) {}
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
/// // Define associated types for a counter block
/// struct CounterBlockSpec;
///
/// impl BlockSpecAssociatedTypes for CounterBlockSpec {
///     type Input = NoInput;     // No input needed
///     type Output = CountOutput; // Outputs a count
///     type State = i32;          // Internal counter state
/// }
/// ```
pub trait BlockSpecAssociatedTypes {
    type Input: BlockInput;
    type Output: BlockOutput;
    type State;
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
/// use block_traits::*;
/// use registry::*;
///
/// // Define wrapper types that implement the required traits
/// #[derive(Clone)]
/// struct IntInput { value: i32 }
///
/// #[derive(Clone)]
/// struct IntOutput { result: i32 }
///
/// // Mock implementations
/// struct IntInputKeys;
/// struct IntOutputKeys;
///
/// impl InputKeys<IntInput> for IntInputKeys {
///     type ReaderType = MockReader<IntInput>;
///     fn reader(&self, _registry: &Registry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: IntInput { value: 21 } })
///     }
/// }
///
/// impl OutputKeys<IntOutput> for IntOutputKeys {
///     type WriterType = MockWriter<IntOutput>;
///     fn writer(&self, _registry: &Registry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///     fn register(&self, _registry: &mut Registry) {}
/// }
///
/// impl BlockInput for IntInput {
///     type Keys = IntInputKeys;
/// }
///
/// impl BlockOutput for IntOutput {
///     type Keys = IntOutputKeys;
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
/// // A simple doubler block that multiplies input by 2
/// struct DoublerBlock;
///
/// impl BlockSpecAssociatedTypes for DoublerBlock {
///     type Input = IntInput;
///     type Output = IntOutput;
///     type State = (); // No state needed
/// }
///
/// impl BlockSpec for DoublerBlock {
///     fn init_state(&self) -> Self::State {
///         () // No initialization needed
///     }
///     
///     fn execute(
///         &self,
///         _context: &ExecutionContext,
///         input: Self::Input,
///         state: &Self::State,
///     ) -> (Self::Output, Self::State) {
///         (IntOutput { result: input.value * 2 }, *state)
///     }
/// }
/// ```
pub trait BlockSpec: BlockSpecAssociatedTypes {
    fn init_state(&self) -> Self::State;
    fn execute(
        &self,
        context: &ExecutionContext,
        input: Self::Input,
        state: &Self::State,
    ) -> (Self::Output, Self::State);

    fn register_outputs(
        &self,
        registry: &mut registry::Registry,
        out_keys: &<Self::Output as BlockOutput>::Keys,
    ) {
        <<Self::Output as BlockOutput>::Keys as registry::OutputKeys<Self::Output>>::register(
            out_keys, registry,
        )
    }

    /// Wire the block to the registry
    fn wire(
        self,
        registry: &registry::Registry,
        in_keys: &<Self::Input as BlockInput>::Keys,
        out_keys: &<Self::Output as BlockOutput>::Keys,
    ) -> Result<WrappedBlock<Self>, registry::RegistryError>
    where
        Self: Sized,
    {
        use registry::{InputKeys, OutputKeys};

        // Create readers/writers that capture the Rc references
        let input_reader = in_keys.reader(registry)?;
        let output_writer = out_keys.writer(registry)?;

        let state = self.init_state();

        Ok(WrappedBlock {
            block: self,
            input_reader,
            output_writer,
            state,
        })
    }

    /// Declare and wire in one step
    fn declare_and_wire(
        self,
        registry: &mut registry::Registry,
        in_keys: &<Self::Input as BlockInput>::Keys,
        out_keys: &<Self::Output as BlockOutput>::Keys,
    ) -> Result<WrappedBlock<Self>, registry::RegistryError>
    where
        Self: Sized,
    {
        self.register_outputs(registry, out_keys);
        self.wire(registry, in_keys, out_keys)
    }
}

/// Trait for executable blocks.
///
/// This is the runtime interface for blocks that have been wired to the registry.
/// The `execute` method is called to run the block's logic.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
///
/// struct SimpleBlock {
///     counter: i32,
/// }
///
/// impl Block for SimpleBlock {
///     fn execute(&mut self, _context: &ExecutionContext) {
///         self.counter += 1;
///         println!("Block executed {} times", self.counter);
///     }
/// }
/// ```
pub trait Block {
    fn execute(&mut self, context: &ExecutionContext);
}

/// Execution context passed to blocks during execution.
///
/// Contains runtime information that blocks may need during execution.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
///
/// let context = ExecutionContext { time: 1234567890 };
/// assert_eq!(context.time, 1234567890);
/// ```
pub struct ExecutionContext {
    pub time: u64,
}

pub struct WrappedBlock<B: BlockSpec> {
    pub block: B,
    pub input_reader: <<B::Input as BlockInput>::Keys as registry::InputKeys<B::Input>>::ReaderType,
    pub output_writer:
        <<B::Output as BlockOutput>::Keys as registry::OutputKeys<B::Output>>::WriterType,
    pub state: B::State,
}

impl<B: BlockSpec> WrappedBlock<B> {
    pub fn new(
        block: B,
        input_reader: <<B::Input as BlockInput>::Keys as registry::InputKeys<B::Input>>::ReaderType,
        output_writer: <<B::Output as BlockOutput>::Keys as registry::OutputKeys<
            B::Output,
        >>::WriterType,
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

impl<B: BlockSpec> Block for WrappedBlock<B> {
    fn execute(&mut self, context: &ExecutionContext) {
        use registry::{Reader, Writer};
        let input = self.input_reader.read();
        let (output, new_state) = self.block.execute(context, input, &self.state);
        self.output_writer.write(&output);
        self.state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use registry::*;
    use std::cell::RefCell;

    // Test implementations for unit tests
    #[derive(Clone, Debug, PartialEq)]
    struct TestInput {
        value: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestOutput {
        result: i32,
    }

    struct TestInputKeys {
        value: i32,
    }

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

    impl InputKeys<TestInput> for TestInputKeys {
        type ReaderType = MockReader<TestInput>;

        fn reader(&self, _registry: &Registry) -> Result<Self::ReaderType, RegistryError> {
            Ok(MockReader {
                data: TestInput { value: self.value },
            })
        }
    }

    impl OutputKeys<TestOutput> for TestOutputKeys {
        type WriterType = MockWriter<TestOutput>;

        fn writer(&self, _registry: &Registry) -> Result<Self::WriterType, RegistryError> {
            Ok(MockWriter {
                written: RefCell::new(None),
            })
        }

        fn register(&self, _registry: &mut Registry) {
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
    }

    impl BlockSpec for DoublerBlock {
        fn init_state(&self) -> Self::State {
            0
        }

        fn execute(
            &self,
            _context: &ExecutionContext,
            input: Self::Input,
            state: &Self::State,
        ) -> (Self::Output, Self::State) {
            let output = TestOutput {
                result: input.value * 2,
            };
            (output, state + 1)
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

        let (output, new_state) = block.execute(&context, input, &state);

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
            let (output, new_state) = block.execute(&context, input.clone(), &state);
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

        let wrapped = WrappedBlock::new(block, reader, writer);
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

        let mut wrapped = WrappedBlock::new(block, reader, writer);
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

        let mut wrapped = WrappedBlock::new(block, reader, writer);
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
    }

    impl BlockSpec for AccumulatorBlock {
        fn init_state(&self) -> Self::State {
            0
        }

        fn execute(
            &self,
            _context: &ExecutionContext,
            input: Self::Input,
            state: &Self::State,
        ) -> (Self::Output, Self::State) {
            let new_state = state + input.value;
            let output = TestOutput { result: new_state };
            (output, new_state)
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
            let (output, new_state) = block.execute(&context, input, &state);
            assert_eq!(output.result, expected);
            assert_eq!(new_state, expected);
            state = new_state;
        }
    }
}
