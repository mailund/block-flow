use super::*;
use block_traits::Block;

#[derive(Debug)]
pub enum ReadBlocksError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for ReadBlocksError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ReadBlocksError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Reads blocks into a vector of BlockType from a JSON string.
/// The enum preserves type information for each block.
pub fn read_blocktypes_from_json_string(json: &str) -> Result<Vec<BlockType>, serde_json::Error> {
    serde_json::from_str::<Vec<BlockType>>(json)
}

/// Reads blocks into a vector of WeaveNode from a JSON string.
/// The returned nodes are type-erased andcan be weaved into a graph.
pub fn read_blocks_from_json_string(
    json: &str,
) -> Result<Vec<Box<dyn WeaveNode<Block>>>, serde_json::Error> {
    let block_types = read_blocktypes_from_json_string(json)?;
    let blocks = block_types.iter().map(|bt| bt.as_weave_block()).collect();
    Ok(blocks)
}

pub fn read_blocktypes_from_json_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<BlockType>, ReadBlocksError> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str::<Vec<BlockType>>(&buf)?)
}
