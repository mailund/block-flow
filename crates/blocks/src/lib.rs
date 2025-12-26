use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use block_macros::*;
use block_traits::{
    BlockEmbedding, BlockPackage, BlockSpec, ContractDeps, ExecuteTrait, ExecutionContextTrait,
    SlotIntent,
};

pub mod after;
pub mod block_io;
pub mod delete;
pub mod simple_order;
pub mod sniper;

pub use after::AfterBlock;
pub use block_io::*;
pub use delete::DeleteBlock;
pub use simple_order::SimpleOrderBlock;
pub use sniper::SniperBlock;

use channels::ChannelKeys;
use weave::WeaveNode;

macro_rules! define_block_type {
    ( $( $variant:ident => $block_ty:path ),+ $(,)? ) => {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "type", content = "data")]
        pub enum BlockTypes {
            $(
                $variant(BlockPackage<$block_ty>),
            )+
        }

        pub enum BlockEmbeddings {
            $(
                $variant(BlockEmbedding<$block_ty>),
            )+
        }

        // Getting From<> for all the wrapped variants
        $(
            impl From<BlockPackage<$block_ty>> for BlockTypes {
                fn from(pkg: BlockPackage<$block_ty>) -> Self {
                    BlockTypes::$variant(pkg)
                }
            }

            impl From<BlockEmbedding<$block_ty>> for BlockEmbeddings {
                fn from(embedded: BlockEmbedding<$block_ty>) -> Self {
                    BlockEmbeddings::$variant(embedded)
                }
            }
        )+

        impl WeaveNode<BlockEmbeddings> for BlockTypes {
            fn input_channels(&self) -> Vec<String> {
                match self {
                    $(
                        BlockTypes::$variant(pkg) => pkg.input_keys.channel_names(),
                    )+
                }
            }
            fn output_channels(&self) -> Vec<String> {
                match self {
                    $(
                        BlockTypes::$variant(pkg) => pkg.output_keys.channel_names(),
                    )+
                }
            }

            fn weave(
                &self,
                channels: &mut ::channels::ChannelRegistry,
            ) -> Result<BlockEmbeddings, channels::RegistryError> {
                match self {
                    $(
                        BlockTypes::$variant(pkg) => Ok(BlockEmbeddings::$variant(pkg.weave(channels)?)),
                    )+
                }
            }
        }

        impl ContractDeps for BlockEmbeddings {
            fn contract_deps(&self) -> Vec<::trade_types::Contract> {
                match self {
                    $(
                        BlockEmbeddings::$variant(embedded) => embedded.contract_deps(),
                    )+
                }
            }
        }

        impl<C: ExecutionContextTrait> ExecuteTrait<C> for BlockEmbeddings {
            fn execute(&self, ctx: &C) -> Option<Vec<SlotIntent>>{
                match self {
                    $(
                        BlockEmbeddings::$variant(embedded) => embedded.execute(ctx),
                    )+
                }
            }
        }
    };
}

define_block_type!(
    After => after::AfterBlock,
    Delete => delete::DeleteBlock,
    SimpleOrder => simple_order::SimpleOrderBlock,
);

#[cfg(test)]
mod test {
    use super::*;
    use block_traits::*;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use trade_types::{Cents, Contract, Price, Side};

    pub struct OrderBook;

    impl block_traits::execution_context::OrderBookTrait for OrderBook {
        fn top_of_side(&self, _side: Side) -> Option<Price> {
            // Dummy implementation
            Some(Price::from(Cents(100)))
        }
    }

    pub struct ExecutionContext {
        pub time: u64,
    }

    impl ExecutionContextTrait for ExecutionContext {
        type OrderBook = OrderBook;
        fn time(&self) -> u64 {
            self.time
        }
        fn get_order_book(&self, _contract: &Contract) -> Option<OrderBook> {
            // Mock implementation
            Some(OrderBook {})
        }
        fn get_position(
            &self,
            _block_id: u32,
            _contract: &Contract,
        ) -> Option<trade_types::Quantity> {
            // mock position
            None
        }
    }

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "blocks_test_{}_{}_{}.json",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    #[test]
    fn block_type_serialization_roundtrip_all_variants() {
        // After
        type AfterInKey =
            <<AfterBlock as BlockSpecAssociatedTypes>::Input as block_traits::BlockInput>::Keys;
        type AfterOutKey =
            <<AfterBlock as BlockSpecAssociatedTypes>::Output as block_traits::BlockOutput>::Keys;
        type AfterInit = <AfterBlock as BlockSpecAssociatedTypes>::InitParameters;

        let after_pkg = BlockPackage::<after::AfterBlock>::new(
            AfterInKey {},
            AfterOutKey {
                is_after: "output_is_after".to_string(),
            },
            AfterInit { time: 123 },
            None,
        );

        // Delete
        type DeleteInKey =
            <<DeleteBlock as BlockSpecAssociatedTypes>::Input as block_traits::BlockInput>::Keys;
        type DeleteOutKey =
            <<DeleteBlock as BlockSpecAssociatedTypes>::Output as block_traits::BlockOutput>::Keys;
        type DeleteInit = <DeleteBlock as BlockSpecAssociatedTypes>::InitParameters;

        let delete_pkg = BlockPackage::<delete::DeleteBlock>::new(
            DeleteInKey {
                should_delete: "should_delete".to_string(),
            },
            DeleteOutKey {},
            DeleteInit {},
            None,
        );

        // SimpleOrder
        type SimpleInKey = <<SimpleOrderBlock as BlockSpecAssociatedTypes>::Input as block_traits::BlockInput>::Keys;
        type SimpleOutKey = <<SimpleOrderBlock as BlockSpecAssociatedTypes>::Output as block_traits::BlockOutput>::Keys;
        type SimpleInit = <SimpleOrderBlock as BlockSpecAssociatedTypes>::InitParameters;

        let contract = trade_types::Contract::new("TEST");
        let simple_pkg = BlockPackage::<simple_order::SimpleOrderBlock>::new(
            SimpleInKey {
                should_execute: "should_execute".to_string(),
            },
            SimpleOutKey {},
            SimpleInit {
                contract: contract.clone(),
                side: trade_types::Side::Buy,
                price: trade_types::Price::from(trade_types::Cents(100)),
                quantity: trade_types::Quantity::from(trade_types::Kw(1)),
            },
            None,
        );

        let blocks = vec![
            BlockTypes::After(after_pkg),
            BlockTypes::Delete(delete_pkg),
            BlockTypes::SimpleOrder(simple_pkg),
        ];

        let serialized = serde_json::to_string(&blocks).unwrap();
        let deserialized: Vec<BlockTypes> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.len(), 3);

        match &deserialized[0] {
            BlockTypes::After(pkg) => {
                assert_eq!(pkg.output_keys.is_after, "output_is_after");
                assert_eq!(pkg.init_params.time, 123);
            }
            _ => panic!("Expected After"),
        }
        match &deserialized[1] {
            BlockTypes::Delete(pkg) => {
                assert_eq!(pkg.input_keys.should_delete, "should_delete");
            }
            _ => panic!("Expected Delete"),
        }
        match &deserialized[2] {
            BlockTypes::SimpleOrder(pkg) => {
                assert_eq!(pkg.init_params.contract, contract);
                assert_eq!(pkg.input_keys.should_execute, "should_execute");
            }
            _ => panic!("Expected SimpleOrder"),
        }
    }

    #[test]
    fn deserialize_single_block_from_string() {
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

        let blocks = read_blocktypes_from_json_string(json).unwrap();
        assert_eq!(blocks.len(), 1);

        match &blocks[0] {
            BlockTypes::After(summary) => {
                assert_eq!(summary.output_keys.is_after, "output_is_after");
            }
            _ => panic!("Wrong block type"),
        }
    }

    #[test]
    fn deserialize_multiple_blocks_from_string() {
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

        let blocks = read_blocktypes_from_json_string(json).unwrap();
        assert_eq!(blocks.len(), 2);

        match &blocks[0] {
            BlockTypes::After(summary) => {
                assert_eq!(summary.init_params.time, 1);
                assert_eq!(summary.output_keys.is_after, "is_after");
            }
            _ => panic!("Expected After block"),
        }

        match &blocks[1] {
            BlockTypes::Delete(summary) => {
                assert_eq!(summary.input_keys.should_delete, "is_after");
            }
            _ => panic!("Expected Delete block"),
        }
    }

    #[test]
    fn read_blocks_from_json_string_produces_weave_nodes_for_all_variants() {
        // Include all variants to cover BlockType::as_weave_node match arms.
        let json = r#"
        [
            {
                "type": "After",
                "data": {
                    "input_keys": {},
                    "output_keys": { "is_after": "is_after" },
                    "init_params": { "time": 5 }
                }
            },
            {
                "type": "Delete",
                "data": {
                    "input_keys": { "should_delete": "is_after" },
                    "output_keys": {},
                    "init_params": null
                }
            },
            {
                "type": "SimpleOrder",
                "data": {
                    "input_keys": { "should_execute": "should_execute" },
                    "output_keys": {},
                    "init_params": {
                        "contract": "TEST",
                        "side": "Buy",
                        "price": { "cents": 100 },
                        "quantity": { "kw": 10 }
                    }
                }
            }
        ]
        "#;

        let nodes = read_blocktypes_from_json_string(json).unwrap();
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn weave_and_execute_end_to_end_after_then_delete() {
        // This executes the type-erased blocks, which drives coverage through:
        // - BlockType -> WeaveNode
        // - keys register/writer/reader
        // - block_traits type_erasure execution path
        let json = r#"
        [
            {
                "type": "After",
                "data": {
                    "input_keys": {},
                    "output_keys": { "is_after": "is_after" },
                    "init_params": { "time": 10 }
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

        let nodes = read_blocktypes_from_json_string(json).unwrap();
        let mut registry = channels::ChannelRegistry::default();

        // Provide inputs required by the After block (it has no input keys, so none needed).
        // Execute After first: it should write is_after (bool) to channel "is_after".
        let after_block = nodes[0].weave(&mut registry).unwrap();
        let ctx = ExecutionContext { time: 11 };
        after_block.execute(&ctx);

        // Now Delete reads should_delete from "is_after" (bool channel). It just prints, but
        // executing it ensures channel reading path is exercised.
        let delete_block = nodes[1].weave(&mut registry).unwrap();
        delete_block.execute(&ctx);

        // Sanity check the produced channel exists and is true.
        let cell = registry.get::<bool>("is_after").unwrap();
        assert!(*cell.borrow());
    }

    #[test]
    fn deserialize_invalid_block_type_fails() {
        let json = r#"
        [
            { "type": "DoesNotExist", "data": {} }
        ]
        "#;

        let result = read_blocktypes_from_json_string(json);
        assert!(result.is_err());
    }

    #[test]
    fn read_blocktypes_from_json_string_invalid_json_fails() {
        // Malformed JSON to cover serde_json error path from from_str.
        let bad = r#"[{ "type": "After", "data": { "#;
        let err = read_blocktypes_from_json_string(bad).unwrap_err();
        // Just assert it's an error; exact message can vary by serde_json version.
        let _ = err.to_string();
    }

    #[test]
    fn read_blocktypes_from_json_file_success() {
        let json = r#"
        [
            {
                "type": "After",
                "data": {
                    "input_keys": {},
                    "output_keys": { "is_after": "is_after" },
                    "init_params": { "time": 1 }
                }
            }
        ]
        "#;

        let path = tmp_path("ok");
        fs::write(&path, json).unwrap();

        let blocks = read_blocktypes_from_json_file(&path).unwrap();
        assert_eq!(blocks.len(), 1);

        // cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn read_blocktypes_from_json_file_missing_file_is_io_error() {
        let path = tmp_path("missing");
        // Ensure it doesn't exist
        let _ = fs::remove_file(&path);

        let err = read_blocktypes_from_json_file(&path).unwrap_err();
        match err {
            ReadBlocksError::Io(_) => {}
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn read_blocktypes_from_json_file_bad_json_is_json_error() {
        let path = tmp_path("badjson");
        fs::write(&path, "not json").unwrap();

        let err = read_blocktypes_from_json_file(&path).unwrap_err();
        match err {
            ReadBlocksError::Json(_) => {}
            _ => panic!("Expected Json error"),
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn read_blocks_error_from_conversions_are_covered() {
        // Explicitly cover both From impls without relying on IO/serde formatting.
        let io_err = io::Error::other("x");
        let e: ReadBlocksError = io_err.into();
        match e {
            ReadBlocksError::Io(_) => {}
            _ => panic!("expected Io"),
        }

        let serde_err: serde_json::Error =
            serde_json::from_str::<Vec<BlockTypes>>("not json").unwrap_err();
        let e: ReadBlocksError = serde_err.into();
        match e {
            ReadBlocksError::Json(_) => {}
            _ => panic!("expected Json"),
        }
    }
}
