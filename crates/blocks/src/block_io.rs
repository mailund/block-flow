use super::*;

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
pub fn read_blocktypes_from_json_string(json: &str) -> Result<Vec<BlockTypes>, serde_json::Error> {
    serde_json::from_str::<Vec<BlockTypes>>(json)
}

pub fn read_blocktypes_from_json_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<BlockTypes>, ReadBlocksError> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str::<Vec<BlockTypes>>(&buf)?)
}
