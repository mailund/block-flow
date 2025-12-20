#[cfg(test)]
mod tests {
    use block_traits::block_weave::BlockSerializationPackage;
    use block_traits::Block;
    use blocks::{AfterBlock, SimpleOrderBlock};
    use channels::ChannelRegistry;
    use trade_types::Contract;
    use weave::*;

    #[test]
    fn weave_after_and_simple_order() {
        let after_node = BlockSerializationPackage::<AfterBlock> {
            input_keys: blocks::after::InputKeys {},
            output_keys: blocks::after::OutputKeys {
                is_after: "after_output".to_string(),
            },
            init_params: blocks::after::InitParams { time: 42 },
        };
        // SimpleOrderBlock expects InitParams { contract: Contract }
        let order_node = BlockSerializationPackage::<SimpleOrderBlock> {
            input_keys: blocks::simple_order::InputKeys {
                should_execute: "after_output".to_string(),
            },
            output_keys: blocks::simple_order::OutputKeys {},
            init_params: blocks::simple_order::InitParams {
                contract: Contract::new("ABC-123"),
            },
        };
        let mut registry = ChannelRegistry::default();
        let nodes: Vec<Box<dyn WeaveNode<Block>>> =
            vec![Box::new(after_node), Box::new(order_node)];
        let result = weave_nodes(nodes, &mut registry);
        assert!(
            result.is_ok(),
            "weave_nodes should succeed for AfterBlock -> SimpleOrderBlock"
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
    }
}
