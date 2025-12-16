use super::*;
use serialization_macros::SerializableStruct;

#[derive(serde::Serialize, serde::Deserialize, SerializableStruct)]
pub struct BlockSerializationSummary<BSpec: BlockSpec> {
    pub input_keys: <BSpec::Input as BlockInput>::Keys,
    pub output_keys: <BSpec::Output as BlockOutput>::Keys,
    pub init_params: BSpec::InitParameters,
}

impl<BSpec: BlockSpec> BlockSerializationSummary<BSpec> {}

pub struct BlockSerialisation;

impl BlockSerialisation {
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
