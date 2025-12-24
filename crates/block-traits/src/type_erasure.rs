use super::*;
use intents::SlotIntent;

pub(super) struct EncapsulatedBlock<B: BlockSpec> {
    pub block: B,
    pub input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
    pub output_writer:
        <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    pub state_cell: std::cell::RefCell<B::State>,
}

impl<B: BlockSpec> EncapsulatedBlock<B> {
    pub fn new(
        block: B,
        input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
        output_writer:
            <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    ) -> Self {
        let init_state = block.init_state();
        let state_cell = std::cell::RefCell::new(init_state);
        Self {
            block,
            input_reader,
            output_writer,
            state_cell,
        }
    }
}

impl<B: BlockSpec> BlockTrait for EncapsulatedBlock<B> {
    fn block_id(&self) -> u32 {
        self.block.block_id()
    }

    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        self.block.contract_deps()
    }

    fn execute(&self, context: &ExecutionContext) -> Option<Vec<SlotIntent>> {
        use crate::intents::BlockIntents;

        let input = self.input_reader.read();
        let old_state = self.state_cell.borrow();

        let (output, new_state, intents) = self.block.execute(context, input, &old_state)?;

        drop(old_state); // Release borrow before mutable borrow
        self.output_writer.write(&output);
        *self.state_cell.borrow_mut() = new_state;

        let slot_intents = intents.as_slot_intents(self.block.block_id());
        Some(slot_intents)
    }
}

impl Block {
    pub fn new<B: BlockSpec + 'static>(
        block: B,
        input_reader: <<B::Input as BlockInput>::Keys as channels::InputKeys<B::Input>>::ReaderType,
        output_writer:
            <<B::Output as BlockOutput>::Keys as channels::OutputKeys<B::Output>>::WriterType,
    ) -> Self {
        let encapsulated = EncapsulatedBlock::new(block, input_reader, output_writer);
        Self {
            block: Box::new(encapsulated),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExecutionContext;
    use block_macros::*;
    use channels::{InputKeys, OutputKeys};

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
        let block = TestBlock { block_id: 1 };
        let mut registry = channels::ChannelRegistry::default();

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        // Output channels should be registered before creating a writer.
        assert!(out_keys.register(&mut registry).is_ok());

        // Insert manual key manually as they don't support registration.
        registry.put("in", 0i32);

        let reader = in_keys.reader(&registry).unwrap();
        let writer = out_keys.writer(&registry).unwrap();

        let enc = EncapsulatedBlock::new(block, reader, writer);
        let _ = enc.state_cell.borrow();
    }

    #[test]
    fn type_erased_block_execute_writes_output_and_returns_intents() {
        let block = TestBlock { block_id: 42 };
        let mut registry = channels::ChannelRegistry::default();

        // Put the input FIELD value (i32) into the channel used by InputKeys.x
        registry.put("in", 10i32);

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        assert!(out_keys.register(&mut registry).is_ok());

        let reader = in_keys.reader(&registry).unwrap();
        let writer = out_keys.writer(&registry).unwrap();

        let enc = EncapsulatedBlock::new(block, reader, writer);
        let ctx = ExecutionContext { time: 0 };

        let intents = enc.execute(&ctx).unwrap();
        assert_eq!(enc.block_id(), 42);
        assert!(intents.is_empty());

        // Output channel stores the FIELD type (i32), not Output struct.
        let cell = registry.get::<i32>("out").unwrap();
        let out = cell.borrow();
        assert_eq!(*out, 20);
    }

    #[test]
    fn block_wrapper_delegates_correctly() {
        let block = TestBlock { block_id: 99 };
        let mut registry = channels::ChannelRegistry::default();

        registry.put("in", 3i32);

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        assert!(out_keys.register(&mut registry).is_ok());

        let reader = in_keys.reader(&registry).unwrap();
        let writer = out_keys.writer(&registry).unwrap();

        let block = Block::new(block, reader, writer);

        let ctx = ExecutionContext { time: 1 };

        assert_eq!(block.block_id(), 99);

        block.execute(&ctx);

        let cell = registry.get::<i32>("out").unwrap();
        let out = cell.borrow();
        assert_eq!(*out, 6);
    }

    #[test]
    fn state_is_updated_across_multiple_executes() {
        let block = TestBlock { block_id: 5 };
        let mut registry = channels::ChannelRegistry::default();

        registry.put("in", 4i32);

        let in_keys = input_keys("in");
        let out_keys = output_keys("out");

        assert!(out_keys.register(&mut registry).is_ok());

        let reader = in_keys.reader(&registry).unwrap();
        let writer = out_keys.writer(&registry).unwrap();

        let enc = EncapsulatedBlock::new(block, reader, writer);
        let ctx = ExecutionContext { time: 0 };

        enc.execute(&ctx);
        enc.execute(&ctx);

        let cell = registry.get::<i32>("out").unwrap();
        let out = cell.borrow();
        assert_eq!(*out, 8);
    }

    use super::test_types::*;
    use std::cell::RefCell;

    #[test]
    fn test_wrapped_block_execute() {
        let block = DoublerBlock;
        let reader = MockReader {
            data: TestInput { value: 15 },
        };
        let writer = MockWriter {
            written: RefCell::new(None),
        };

        let wrapped = EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 200 };

        wrapped.execute(&context);

        let written_data = wrapped.output_writer.written.borrow();
        assert!(written_data.is_some());
        assert_eq!(written_data.as_ref().unwrap().result, 30); // 15 * 2

        assert_eq!(*wrapped.state_cell.borrow(), TestState { acc: 1 });
    }

    #[test]
    fn test_multiple_wrapped_block_executions() {
        let block = DoublerBlock;
        let reader = MockReader {
            data: TestInput { value: 3 },
        };
        let writer = MockWriter {
            written: RefCell::new(None),
        };

        let wrapped = type_erasure::EncapsulatedBlock::new(block, reader, writer);
        let context = ExecutionContext { time: 300 };

        for expected_state in 1..=5 {
            wrapped.execute(&context);
            assert_eq!(
                *wrapped.state_cell.borrow(),
                TestState {
                    acc: expected_state
                }
            );

            let written_data = wrapped.output_writer.written.borrow();
            assert_eq!(written_data.as_ref().unwrap().result, 6); // 3 * 2
        }
    }
}
