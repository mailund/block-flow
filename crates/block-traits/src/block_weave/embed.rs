use super::*;
use channels::{InputKeys, OutputKeys};
use weave::EmbeddedNode;

/// Encapsulates a block along with its input reader, output writer, and state cell
/// to provide a type-erased block implementation. The BlockEmbedding is a block with its
/// serialisation connections established, ready to be used as an execution context.
pub struct BlockEmbedding<B: BlockSpec> {
    package: BlockPackage<B>,
    block: B,
    in_reader: block_keys::InReader<B>,
    out_writer: block_keys::OutWriter<B>,
    state_cell: std::cell::RefCell<B::State>,
}

impl<B: BlockSpec> BlockEmbedding<B> {
    pub fn new_from_package(
        package: &BlockPackage<B>,
        registry: &mut channels::ChannelRegistry,
    ) -> Result<Self, channels::RegistryError> {
        let package = package.clone();

        let in_reader = package.input_keys.reader(registry)?;
        let out_writer = package.output_keys.writer(registry)?;

        let block = B::new_from_init_params(&package.init_params);
        let state = match &package.state {
            Some(state) => state.clone(),
            None => block.init_state(),
        };
        let state_cell = std::cell::RefCell::new(state);

        let embedded = Self {
            package,
            block,
            in_reader,
            out_writer,
            state_cell,
        };

        Ok(embedded)
    }

    pub fn extract_package(&self) -> BlockPackage<B> {
        // Take the input/output from the stored package
        // but return the current state from the state cell.
        BlockPackage {
            input_keys: self.package.input_keys.clone(),
            output_keys: self.package.output_keys.clone(),
            init_params: self.package.init_params.clone(),
            state: Some(self.state_cell.borrow().clone()),
        }
    }
}

impl<B> EmbeddedNode<BlockPackage<B>> for BlockEmbedding<B>
where
    B: BlockSpec + 'static,
{
    fn extract_package(&self) -> BlockPackage<B> {
        BlockEmbedding::<B>::extract_package(self)
    }
}

impl<B> ContractDeps for BlockEmbedding<B>
where
    B: BlockSpec,
{
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.block.contract_deps()
    }
}

/// Implement BlockTrait for BlockPackage to allow type-erased execution.
impl<B, C> ExecuteTrait<C> for BlockEmbedding<B>
where
    B: BlockSpec,
    C: ExecutionContextTrait,
{
    fn execute(&self, context: &C) -> Option<Vec<SlotIntent>> {
        use crate::intents::BlockIntents;

        let input = self.in_reader.read();
        let old_state = self.state_cell.borrow();

        let (output, new_state, intents) = self.block.execute(context, input, &old_state)?;

        drop(old_state); // Release borrow before mutable borrow
        self.out_writer.write(&output);
        *self.state_cell.borrow_mut() = new_state;

        let slot_intents = intents.as_slot_intents(self.block.block_id());
        Some(slot_intents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_macros::*;
    use channels::OutputKeys;
    use trade_types::{Cents, Contract, Price, Side};

    pub struct OrderBook;

    impl execution_context::OrderBookTrait for OrderBook {
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

    // ---------------- Test Block ----------------
    mod test_block {
        use super::*;
        make_defaults!(state, init_params);

        #[input]
        pub struct Input {
            pub x: i32,
        }

        #[output]
        pub struct Output {
            pub y: i32,
        }

        #[block]
        pub struct TestBlock {
            pub block_id: u32,
        }

        impl BlockSpec for TestBlock {
            fn block_id(&self) -> u32 {
                self.block_id
            }

            fn new_from_init_params(_: &InitParams) -> Self {
                TestBlock { block_id: 77 }
            }

            fn init_state(&self) -> State {
                State
            }

            #[execute]
            fn execute(&self, input: Input) -> Output {
                Output { y: input.x * 2 }
            }
        }
    }

    use test_block::TestBlock;

    fn input_keys(name: &str) -> test_block::InputKeys {
        test_block::InputKeys {
            x: name.to_string(),
        }
    }

    fn output_keys(name: &str) -> test_block::OutputKeys {
        test_block::OutputKeys {
            y: name.to_string(),
        }
    }

    // ---------------- Tests ----------------

    #[test]
    fn encapsulated_block_new_initializes_state() {
        let mut registry = channels::ChannelRegistry::default();

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        // Output channels should be registered before creating a writer.
        assert!(out_keys.register(&mut registry).is_ok());

        // Insert manual key manually as they don't support registration.
        registry.put("in", 0i32);

        let package =
            BlockPackage::<TestBlock>::new(in_keys, out_keys, test_block::InitParams {}, None);
        let enc = package.weave(&mut registry).unwrap();

        let _ = enc.state_cell.borrow();
    }

    #[test]
    fn type_erased_block_execute_writes_output_and_returns_intents() {
        let mut registry = channels::ChannelRegistry::default();

        // Put the input FIELD value (i32) into the channel used by InputKeys.x
        registry.put("in", 10i32);

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        assert!(out_keys.register(&mut registry).is_ok());

        let package =
            BlockPackage::<TestBlock>::new(in_keys, out_keys, test_block::InitParams {}, None);
        let enc = package.weave(&mut registry).unwrap();
        let ctx = ExecutionContext { time: 0 };

        let intents = enc.execute(&ctx).unwrap();
        assert!(intents.is_empty());

        // Output channel stores the FIELD type (i32), not Output struct.
        let cell = registry.get::<i32>("out").unwrap();
        let out = cell.borrow();
        assert_eq!(*out, 20);
    }

    #[test]
    fn block_wrapper_delegates_correctly() {
        let mut registry = channels::ChannelRegistry::default();

        registry.put("in", 3i32);

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        assert!(out_keys.register(&mut registry).is_ok());

        let package =
            BlockPackage::<TestBlock>::new(in_keys, out_keys, test_block::InitParams {}, None);
        let enc = package.weave(&mut registry).unwrap();
        let ctx = ExecutionContext { time: 1 };

        enc.execute(&ctx);

        let cell = registry.get::<i32>("out").unwrap();
        let out = cell.borrow();
        assert_eq!(*out, 6);
    }
}
