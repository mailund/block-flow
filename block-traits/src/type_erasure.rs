use super::*;

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
        use ::intents::BlockIntents; // For the as_slice method

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
    pub(crate) block: Box<dyn TypeErasedBlock>,
}

impl Block {
    pub fn new(block: Box<dyn TypeErasedBlock>) -> Self {
        Self { block }
    }

    pub fn execute(&self, context: &ExecutionContext) -> Vec<Intent> {
        self.block.execute(context).as_slice().to_vec()
    }
}
