use block_traits::{Block, BlockInput, BlockOutput, BlockSpec, EncapsulatedBlock};
use channels::{ChannelKeys, InputKeys, OutputKeys, RegistryError};

/// Blocks that have been deserialized and had their
/// concrete types erased for execution in a weave.
pub trait BlockNode {
    fn input_channels(&self) -> Vec<String>;
    fn output_channels(&self) -> Vec<String>;
    fn weave(&self, channels: &mut ::channels::ChannelRegistry) -> Result<Block, RegistryError>;
}

/// Block that has been deserialized (or serialized)
/// before we weave it and erase its concrete type.
#[serialization_macros::serializable_struct]
pub struct BlockSerializationSummary<BSpec: BlockSpec> {
    pub input_keys: <BSpec::Input as BlockInput>::Keys,
    pub output_keys: <BSpec::Output as BlockOutput>::Keys,
    pub init_params: BSpec::InitParameters,
}

impl<BSpec: BlockSpec + 'static> BlockNode for BlockSerializationSummary<BSpec> {
    fn input_channels(&self) -> Vec<String> {
        self.input_keys.channel_names()
    }
    fn output_channels(&self) -> Vec<String> {
        self.output_keys.channel_names()
    }

    fn weave(&self, channels: &mut ::channels::ChannelRegistry) -> Result<Block, RegistryError> {
        self.output_keys.register(channels);

        let block = BSpec::new_from_init_params(&self.init_params);
        let state = block.init_state();

        let input_reader = self.input_keys.reader(channels)?;
        let output_writer = self.output_keys.writer(channels)?;

        let encapsulated = EncapsulatedBlock {
            block,
            input_reader,
            output_writer,
            state_cell: std::cell::RefCell::new(state),
        };

        Ok(Block::new(Box::new(encapsulated)))
    }
}

pub struct BlockSerialisation;

impl BlockSerialisation {
    pub fn new_node<B: BlockSpec>(
        input_keys: <B::Input as BlockInput>::Keys,
        output_keys: <B::Output as BlockOutput>::Keys,
        init_params: B::InitParameters,
    ) -> BlockSerializationSummary<B> {
        BlockSerializationSummary {
            input_keys,
            output_keys,
            init_params,
        }
    }

    pub fn serialize_block<B: BlockSpec, S: ::serialization::StructSerializer>(
        serializer: &S,
        block: BlockSerializationSummary<B>,
    ) -> ::serialization::Result<Vec<u8>> {
        serializer.serialize(&block)
    }

    pub fn deserialize_block<B: BlockSpec, S: ::serialization::StructSerializer>(
        serializer: &S,
        data: &[u8],
    ) -> Result<BlockSerializationSummary<B>, ::serialization::SerializationError> {
        serializer.deserialize::<BlockSerializationSummary<B>>(data)
    }
}
