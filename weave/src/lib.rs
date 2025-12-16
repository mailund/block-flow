use std::collections::{HashMap, HashSet, VecDeque};

use block_traits::{Block, BlockInput, BlockOutput, BlockSpec, EncapsulatedBlock};
use channels::{ChannelKeys, ChannelRegistry, InputKeys, OutputKeys, RegistryError};
use serialization_macros::SerializableStruct;

pub trait BlockNode {
    fn input_channels(&self) -> Vec<String>;
    fn output_channels(&self) -> Vec<String>;
    fn weave(&self, channels: &mut ::channels::ChannelRegistry) -> Result<Block, RegistryError>;
}

#[derive(serde::Serialize, serde::Deserialize, SerializableStruct)]
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
            state,
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

pub fn weave_nodes(
    nodes: Vec<Box<dyn BlockNode>>,
    registry: &mut ChannelRegistry,
) -> Result<Vec<Block>, RegistryError> {
    // Index nodes
    let n = nodes.len();

    // Collect inputs/outputs once (avoid recomputing, and keep ownership of Strings)
    let inputs: Vec<Vec<String>> = nodes.iter().map(|n| n.input_channels()).collect();
    let outputs: Vec<Vec<String>> = nodes.iter().map(|n| n.output_channels()).collect();

    // Map each output channel -> producer node index (error if duplicates)
    let mut producer_of: HashMap<String, usize> = HashMap::new();
    for (i, outs) in outputs.iter().enumerate() {
        for ch in outs {
            if producer_of.insert(ch.clone(), i).is_some() {
                return Err(RegistryError::DuplicateOutputKey(format!("'{ch}'")));
            }
        }
    }

    // Build graph edges producer -> consumer and indegrees
    let mut edges: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    let mut indegree: Vec<usize> = vec![0; n];

    for (consumer, ins) in inputs.iter().enumerate() {
        for ch in ins {
            if let Some(&producer) = producer_of.get(ch) {
                if producer != consumer && edges[producer].insert(consumer) {
                    indegree[consumer] += 1;
                }
            } else {
                // Allow external channels already present in registry, otherwise error.
                // If your registry uses a different API than `has`, change this.
                if !registry.has(ch) {
                    return Err(RegistryError::MissingProducer(format!(
                        "Missing producer for input channel '{ch}' (node index {consumer})"
                    )));
                }
            }
        }
    }

    // Kahn topological sort
    let mut q: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(i, &d)| (d == 0).then_some(i))
        .collect();

    let mut topo: Vec<usize> = Vec::with_capacity(n);
    while let Some(u) = q.pop_front() {
        topo.push(u);
        for &v in edges[u].iter() {
            indegree[v] -= 1;
            if indegree[v] == 0 {
                q.push_back(v);
            }
        }
    }

    if topo.len() != n {
        let cyclic: Vec<usize> = indegree
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| (d > 0).then_some(i))
            .collect();
        return Err(RegistryError::CycleDetected(format!("{cyclic:?}")));
    }

    // Weave in topo order
    let mut blocks = Vec::with_capacity(n);
    for idx in topo {
        blocks.push(nodes[idx].weave(registry)?);
    }

    Ok(blocks)
}
