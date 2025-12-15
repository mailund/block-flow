use registry;

pub trait Block {
    fn execute(&mut self, context: &ExecutionContext);
}

pub struct ExecutionContext {
    pub time: u64,
}

pub trait BlockInput: Sized {
    type Keys: registry::InputKeys<Self>;
}

pub trait BlockOutput: Sized {
    type Keys: registry::OutputKeys<Self>;
}

pub trait BlockSpecAssociatedTypes {
    type Input: BlockInput;
    type Output: BlockOutput;
    type State;
}

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
