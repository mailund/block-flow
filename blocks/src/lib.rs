use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use block_macros::{block, init_params, input, output, state};
use block_traits::{BlockSpec, ExecutionContext};
use weave::BlockSerializationSummary;

mod after;
pub use after::AfterBlock;
mod delete;
pub use delete::DeleteBlock;
mod simple_order;
pub use simple_order::SimpleOrderBlock;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BlockType {
    // FIXME: Not super happy with having a global enum list like this,
    // but it will do for now.
    After(BlockSerializationSummary<after::AfterBlock>),
    Delete(BlockSerializationSummary<delete::DeleteBlock>),
    SimpleOrder(BlockSerializationSummary<simple_order::SimpleOrderBlock>),
}

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

pub fn read_blocks_from_json_str(json: &str) -> Result<Vec<BlockType>, serde_json::Error> {
    serde_json::from_str::<Vec<BlockType>>(json)
}

pub fn read_blocks_from_json_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<BlockType>, ReadBlocksError> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str::<Vec<BlockType>>(&buf)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use block_traits::BlockSpecAssociatedTypes;

    #[test]
    fn test_block_type_serialization() {
        type InKey =
            <<AfterBlock as BlockSpecAssociatedTypes>::Input as block_traits::BlockInput>::Keys;
        type OutKey =
            <<AfterBlock as BlockSpecAssociatedTypes>::Output as block_traits::BlockOutput>::Keys;
        type InitParams = <AfterBlock as BlockSpecAssociatedTypes>::InitParameters;
        let block_summary = BlockSerializationSummary::<after::AfterBlock> {
            input_keys: InKey {},
            output_keys: OutKey {
                is_after: "output_is_after".to_string(),
            },
            init_params: InitParams { time: 0 },
        };
        let block_type = BlockType::After(block_summary);
        let serialized = serde_json::to_string(&block_type).unwrap();
        let deserialized: BlockType = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            BlockType::After(summary) => {
                assert_eq!(summary.output_keys.is_after, "output_is_after");
            }
            _ => panic!("Deserialized to wrong block type"),
        }
    }

    #[test]
    fn test_deserialize_single_block_from_string() {
        let json = r#"
    [
        {
            "type": "After",
            "data": {
                "input_keys": {},
                "output_keys": { "is_after": "output_is_after" },
                "init_params": { "time": 0 }
            }
        }
    ]
    "#;

        let blocks = read_blocks_from_json_str(json).unwrap();
        assert_eq!(blocks.len(), 1);

        match &blocks[0] {
            BlockType::After(summary) => {
                assert_eq!(summary.output_keys.is_after, "output_is_after");
            }
            _ => panic!("Wrong block type"),
        }
    }

    #[test]
    fn test_deserialize_multiple_blocks_from_string() {
        let json = r#"
    [
        {
            "type": "After",
            "data": {
                "input_keys": {},
                "output_keys": { "is_after": "is_after" },
                "init_params": { "time": 1 }
            }
        },
        {
            "type": "Delete",
            "data": {
                "input_keys": { "should_delete": "is_after" },
                "output_keys": {},
                "init_params": null
            }
        }
    ]
    "#;

        let blocks = read_blocks_from_json_str(json).unwrap();
        assert_eq!(blocks.len(), 2);

        match &blocks[0] {
            BlockType::After(summary) => {
                assert_eq!(summary.init_params.time, 1);
                assert_eq!(summary.output_keys.is_after, "is_after");
            }
            _ => panic!("Expected After block"),
        }

        match &blocks[1] {
            BlockType::Delete(summary) => {
                assert_eq!(summary.input_keys.should_delete, "is_after");
            }
            _ => panic!("Expected Delete block"),
        }
    }

    #[test]
    fn test_deserialize_invalid_block_type_fails() {
        let json = r#"
    [
        {
            "type": "DoesNotExist",
            "data": {}
        }
    ]
    "#;

        let result = read_blocks_from_json_str(json);
        assert!(result.is_err());
    }
}
