#[cfg(test)]
mod tests {
    use block_traits::block_weave::BlockPackage;
    use blocks::BlockPackages;
    use blocks::{after::AfterBlock, simple_order::SimpleOrderBlock};
    use channels::ChannelRegistry;
    use trade_types::Contract;
    use weave::*;

    #[test]
    fn weave_after_and_simple_order() {
        let after_node = BlockPackage::<AfterBlock> {
            input_keys: blocks::after::InputKeys {},
            output_keys: blocks::after::OutputKeys {
                is_after: "after_output".to_string(),
            },
            init_params: blocks::after::InitParams { time: 42 },
            state: None,
        };
        // SimpleOrderBlock expects InitParams { contract: Contract }
        let order_node = BlockPackage::<SimpleOrderBlock> {
            input_keys: blocks::simple_order::InputKeys {
                should_execute: "after_output".to_string(),
            },
            output_keys: blocks::simple_order::OutputKeys {},
            init_params: blocks::simple_order::InitParams {
                contract: Contract::new("ABC-123"),
                side: trade_types::Side::Buy,
                price: trade_types::Price::from(trade_types::Cents(100)),
                quantity: trade_types::Quantity::from(trade_types::Kw(1)),
            },
            state: None,
        };
        let mut registry = ChannelRegistry::default();
        let blocks: Vec<BlockPackages> = vec![after_node.into(), order_node.into()];
        let result = weave_nodes(&blocks, &mut registry);
        assert!(
            result.is_ok(),
            "weave_nodes should succeed for AfterBlock -> SimpleOrderBlock"
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
    }
}
