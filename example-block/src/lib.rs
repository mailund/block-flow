use registry::{Registry, RegistryError};
use std::cell::RefCell;
use std::rc::Rc;

/// A simple adder block that adds two numbers
pub struct AdderBlock {
    pub offset: i32,
}

/// Input for the adder block
pub struct AdderInput {
    pub a: i32,
    pub b: i32,
}

/// Output for the adder block
pub struct AdderOutput {
    pub sum: i32,
}

/// State for the adder block (empty for this example)
pub struct AdderState {
    pub call_count: u32,
}

/// Input keys for the adder block
pub struct AdderInKeys {
    pub a: String,
    pub b: String,
}

/// Output keys for the adder block  
pub struct AdderOutKeys {
    pub sum: String,
}

/// Reader that holds direct references to registry values
pub struct AdderInputReader {
    a: Rc<RefCell<i32>>,
    b: Rc<RefCell<i32>>,
}

/// Writer that holds direct references to registry values
pub struct AdderOutputWriter {
    sum: Rc<RefCell<i32>>,
}

impl AdderInput {
    /// Create a reader from the registry using the provided keys
    pub fn reader(
        keys: &AdderInKeys,
        registry: &Registry,
    ) -> Result<AdderInputReader, RegistryError> {
        Ok(AdderInputReader {
            a: registry.get::<i32>(&keys.a)?,
            b: registry.get::<i32>(&keys.b)?,
        })
    }
}

impl AdderOutput {
    /// Create a writer from the registry using the provided keys
    pub fn writer(
        keys: &AdderOutKeys,
        registry: &Registry,
    ) -> Result<AdderOutputWriter, RegistryError> {
        Ok(AdderOutputWriter {
            sum: registry.get::<i32>(&keys.sum)?,
        })
    }
}

/// Wire channels for both input and output, returning reader and writer
pub fn wire_channels(
    in_keys: &AdderInKeys,
    out_keys: &AdderOutKeys,
    registry: &Registry,
) -> Result<(AdderInputReader, AdderOutputWriter), RegistryError> {
    let reader = AdderInput::reader(in_keys, registry)?;
    let writer = AdderOutput::writer(out_keys, registry)?;
    Ok((reader, writer))
}
impl AdderInputReader {
    /// Read input values from the captured references
    pub fn read(&self) -> AdderInput {
        AdderInput {
            a: *self.a.borrow(),
            b: *self.b.borrow(),
        }
    }
}

impl AdderOutputWriter {
    /// Write output values to the captured references
    pub fn write(&self, output: &AdderOutput) {
        *self.sum.borrow_mut() = output.sum;
    }
}

impl AdderBlock {
    /// Constructor
    pub fn new(offset: i32) -> Self {
        Self { offset }
    }

    /// Initialize state
    pub fn init_state(&self) -> AdderState {
        AdderState { call_count: 0 }
    }

    /// Pure execute function
    pub fn execute(&self, input: AdderInput, state: AdderState) -> (AdderOutput, AdderState) {
        let result = input.a + input.b + self.offset;
        let new_state = AdderState {
            call_count: state.call_count + 1,
        };

        let output = AdderOutput { sum: result };
        (output, new_state)
    }

    /// Declare outputs in the registry
    pub fn declare_outputs(&self, registry: &mut Registry, out_keys: &AdderOutKeys) {
        registry.ensure::<i32>(&out_keys.sum);
    }

    /// Wire the block to the registry
    pub fn wire(
        &self,
        registry: &Registry,
        in_keys: &AdderInKeys,
        out_keys: &AdderOutKeys,
    ) -> Result<AdderWiredBlock, RegistryError> {
        // Create readers/writers that capture the Rc references
        let (input_reader, output_writer) = wire_channels(in_keys, out_keys, registry)?;

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
        in_keys: &AdderInKeys,
        out_keys: &AdderOutKeys,
    ) -> Result<AdderWiredBlock, RegistryError> {
        self.declare_outputs(registry, out_keys);
        self.wire(registry, in_keys, out_keys)
    }
}

/// A wired block that can be ticked
pub struct AdderWiredBlock {
    block: AdderBlock,
    input_reader: AdderInputReader,
    output_writer: AdderOutputWriter,
    state: AdderState,
}

impl AdderWiredBlock {
    pub fn tick(&mut self) {
        // Read input from captured references
        let input = self.input_reader.read();

        // Execute the pure function
        let (output, new_state) = self.block.execute(
            input,
            AdderState {
                call_count: self.state.call_count,
            },
        );

        // Write output to captured references
        self.output_writer.write(&output);

        // Update internal state
        self.state = new_state;
    }

    pub fn execute(&mut self) {
        self.tick();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder_block_pure_execution() {
        let block = AdderBlock::new(10);

        let input = AdderInput { a: 5, b: 3 };
        let state = block.init_state();

        let (output, new_state) = block.execute(input, state);

        assert_eq!(output.sum, 18); // 5 + 3 + 10
        assert_eq!(new_state.call_count, 1);
    }

    #[test]
    fn test_adder_block_multiple_calls() {
        let block = AdderBlock::new(0);
        let state = block.init_state();

        let (output1, new_state1) = block.execute(AdderInput { a: 1, b: 2 }, state);
        assert_eq!(output1.sum, 3);
        assert_eq!(new_state1.call_count, 1);

        let (output2, new_state2) = block.execute(AdderInput { a: 10, b: 20 }, new_state1);
        assert_eq!(output2.sum, 30);
        assert_eq!(new_state2.call_count, 2);
    }

    #[test]
    fn test_input_output_readers() {
        let mut registry = Registry::new();
        registry.put("input_a", 7);
        registry.put("input_b", 13);
        registry.put("output_sum", 0);

        let in_keys = AdderInKeys {
            a: "input_a".to_string(),
            b: "input_b".to_string(),
        };

        let out_keys = AdderOutKeys {
            sum: "output_sum".to_string(),
        };

        // Test AdderInput::reader
        let reader = AdderInput::reader(&in_keys, &registry).unwrap();
        let input = reader.read();
        assert_eq!(input.a, 7);
        assert_eq!(input.b, 13);

        // Test AdderOutput::writer
        let writer = AdderOutput::writer(&out_keys, &registry).unwrap();
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

        let in_keys = AdderInKeys {
            a: "input_a".to_string(),
            b: "input_b".to_string(),
        };

        let out_keys = AdderOutKeys {
            sum: "output_sum".to_string(),
        };

        // Declare and wire the block
        let mut wired = block
            .declare_and_wire(&mut registry, &in_keys, &out_keys)
            .unwrap();

        // Execute one tick
        wired.tick();

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

        let in_keys = AdderInKeys {
            a: "a".to_string(),
            b: "b".to_string(),
        };

        let out_keys = AdderOutKeys {
            sum: "sum".to_string(),
        };

        let mut wired = block
            .declare_and_wire(&mut registry, &in_keys, &out_keys)
            .unwrap();

        // First tick
        wired.tick();
        let result = registry.get::<i32>("sum").unwrap();
        assert_eq!(*result.borrow(), 3);

        // Update inputs in registry
        let a_ref = registry.get::<i32>("a").unwrap();
        let b_ref = registry.get::<i32>("b").unwrap();
        *a_ref.borrow_mut() = 10;
        *b_ref.borrow_mut() = 20;

        // Second tick should see updated values
        wired.tick();
        let result = registry.get::<i32>("sum").unwrap();
        assert_eq!(*result.borrow(), 30);
    }
}
