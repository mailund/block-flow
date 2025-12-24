use crate::type_erasure::BlockPackage;

use super::{Block, BlockInput, BlockOutput, BlockSpec};
use channels::{ChannelKeys, InputKeys, OutputKeys, RegistryError};
use weave::WeaveNode;

/// Block that has been deserialized (or serialized)
/// before we weave it and erase its concrete type.
#[serialization_macros::serializable_struct]
pub struct BlockSerializationPackage<BSpec: BlockSpec> {
    pub input_keys: <BSpec::Input as BlockInput>::Keys,
    pub output_keys: <BSpec::Output as BlockOutput>::Keys,
    pub init_params: BSpec::InitParameters,
}

impl<B: BlockSpec> BlockSerializationPackage<B> {
    /// Construct a new serialization/deserialization summary
    /// for a block of type <B> from its input/output keys and
    /// init parameters.
    pub fn new(
        input_keys: <B::Input as BlockInput>::Keys,
        output_keys: <B::Output as BlockOutput>::Keys,
        init_params: B::InitParameters,
    ) -> Self {
        BlockSerializationPackage {
            input_keys,
            output_keys,
            init_params,
        }
    }
}

impl<BSpec: BlockSpec + 'static> WeaveNode<Block> for BlockSerializationPackage<BSpec> {
    fn input_channels(&self) -> Vec<String> {
        self.input_keys.channel_names()
    }
    fn output_channels(&self) -> Vec<String> {
        self.output_keys.channel_names()
    }

    fn weave(&self, channels: &mut ::channels::ChannelRegistry) -> Result<Block, RegistryError> {
        self.output_keys.register(channels)?;

        let block = BSpec::new_from_init_params(&self.init_params);
        let input_reader = self.input_keys.reader(channels)?;
        let output_writer = self.output_keys.writer(channels)?;
        let package = BlockPackage::new_from_reader_writer(block, input_reader, output_writer);

        Ok(package.into())
    }
}

pub fn serialize_block<B: BlockSpec, S: ::serialization::StructSerializer>(
    serializer: &S,
    block: BlockSerializationPackage<B>,
) -> ::serialization::Result<Vec<u8>> {
    serializer.serialize(&block)
}

pub fn deserialize_block<B: BlockSpec, S: ::serialization::StructSerializer>(
    serializer: &S,
    data: &[u8],
) -> Result<BlockSerializationPackage<B>, ::serialization::SerializationError> {
    serializer.deserialize::<BlockSerializationPackage<B>>(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_macros::*;

    // The block-macros expand to ::block_traits::...; inside the block-traits crate
    // we must alias the current crate so that absolute path resolves.
    extern crate self as block_traits;

    // Generate default State/Output/InitParams as needed; Keys are NOT Default.
    make_defaults!(state);

    #[input]
    pub struct Input {
        pub value: i32,
    }

    #[output]
    pub struct Output {
        pub result: i32,
    }

    #[init_params]
    pub struct InitParams {
        pub multiplier: i32,
    }

    #[block]
    pub struct MultiplyBlock {
        pub block_id: u32,
        multiplier: i32,
    }

    impl BlockSpec for MultiplyBlock {
        fn block_id(&self) -> u32 {
            self.block_id
        }

        fn new_from_init_params(params: &InitParams) -> Self {
            MultiplyBlock {
                block_id: 0,
                multiplier: params.multiplier,
            }
        }

        fn init_state(&self) -> State {
            State
        }

        #[execute]
        fn execute(&self, input: Input) -> Output {
            Output {
                result: input.value * self.multiplier,
            }
        }
    }

    fn keys_in(name: &str) -> InputKeys {
        // Keys mirror the struct fields but map them to channel names (Strings).
        InputKeys {
            value: name.to_string(),
        }
    }

    fn keys_out(name: &str) -> OutputKeys {
        OutputKeys {
            result: name.to_string(),
        }
    }

    #[test]
    fn package_new_smoke_and_channel_lists() {
        let pkg = BlockSerializationPackage::<MultiplyBlock>::new(
            keys_in("in"),
            keys_out("out"),
            InitParams { multiplier: 3 },
        );

        assert_eq!(pkg.init_params.multiplier, 3);
        assert_eq!(pkg.input_channels(), vec!["in".to_string()]);
        assert_eq!(pkg.output_channels(), vec!["out".to_string()]);
    }

    #[test]
    fn weave_node_channel_lists_are_correct_for_arbitrary_names() {
        let pkg = BlockSerializationPackage::<MultiplyBlock>::new(
            keys_in("input_chan"),
            keys_out("output_chan"),
            InitParams { multiplier: 2 },
        );

        assert_eq!(pkg.input_channels(), vec!["input_chan".to_string()]);
        assert_eq!(pkg.output_channels(), vec!["output_chan".to_string()]);
    }

    #[test]
    fn serialize_deserialize_roundtrip_preserves_init_params_and_keys() {
        let pkg = BlockSerializationPackage::<MultiplyBlock>::new(
            keys_in("in"),
            keys_out("out"),
            InitParams { multiplier: 7 },
        );

        let ser = ::serialization::structs::JsonStructSerializer::new();

        let bytes = serialize_block::<MultiplyBlock, _>(&ser, pkg).unwrap();
        let restored = deserialize_block::<MultiplyBlock, _>(&ser, &bytes).unwrap();

        assert_eq!(restored.init_params.multiplier, 7);
        assert_eq!(restored.input_channels(), vec!["in".to_string()]);
        assert_eq!(restored.output_channels(), vec!["out".to_string()]);
    }

    #[test]
    fn deserialize_invalid_json_errors() {
        let ser = ::serialization::structs::JsonStructSerializer::new();
        let bad = br#"{ not valid json }"#;

        let res = deserialize_block::<MultiplyBlock, _>(&ser, bad);
        assert!(res.is_err());
    }

    #[test]
    fn weave_returns_error_when_input_channel_is_missing() {
        // This test *intentionally* does not populate the channel registry.
        // We verify the weave function propagates the RegistryError correctly.
        let pkg = BlockSerializationPackage::<MultiplyBlock>::new(
            keys_in("missing_input"),
            keys_out("out"),
            InitParams { multiplier: 10 },
        );

        let mut registry: ::channels::ChannelRegistry = Default::default();
        let res = pkg.weave(&mut registry);

        assert!(res.is_err());
    }
}
