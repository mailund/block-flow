use registry::{InputKeys, OutputKeys, Reader, Registry, Writer};

pub trait Block {
    fn execute(&mut self);
}

pub trait BlockSpec {
    type Input;
    type Output;
    type State;

    type InputKeys: InputKeys<Self::Input>;
    type OutputKeys: OutputKeys<Self::Output>;

    fn init_state(&self) -> Self::State;
    fn execute(&self, input: Self::Input, state: &Self::State) -> (Self::Output, Self::State);

    fn register_outputs(&self, registry: &mut Registry, out_keys: &Self::OutputKeys);
}

pub struct WrappedBlock<B: BlockSpec> {
    pub block: B,
    pub input_reader: <B::InputKeys as InputKeys<B::Input>>::ReaderType,
    pub output_writer: <B::OutputKeys as OutputKeys<B::Output>>::WriterType,
    pub state: B::State,
}

impl<B: BlockSpec> WrappedBlock<B> {
    pub fn new(
        block: B,
        input_reader: <B::InputKeys as InputKeys<B::Input>>::ReaderType,
        output_writer: <B::OutputKeys as OutputKeys<B::Output>>::WriterType,
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
    fn execute(&mut self) {
        let input = self.input_reader.read();
        let (output, new_state) = self.block.execute(input, &self.state);
        self.output_writer.write(&output);
        self.state = new_state;
    }
}
