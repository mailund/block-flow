use block_traits::{Block, BlockTrait};
use std::collections::HashMap;
use std::rc::Rc;
use trade_types::Contract;

/// This is a mock of outbound orders
pub struct Order;

/// A mock actor.
pub struct Actor {
    /// The block encapsulated by this actor.
    /// A block can be a simple block or a composite block,
    /// so in practice the block is usually an execution plan
    /// containing multiple blocks.
    block: Block,
}

impl Actor {
    /// Create a new actor encapsulating the given block.
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn contracts(&self) -> Vec<Contract> {
        self.block.contract_deps()
    }

    pub fn tick(&self, context: &execution_context::ExecutionContext) -> Option<Vec<Order>> {
        let _intents = self.block.execute(context)?;
        // reconcile to orders
        Some(vec![])
    }
}

pub struct ActorController {
    id_to_actors: HashMap<u32, Rc<Actor>>,
    contracts_to_actors: HashMap<Contract, Vec<Rc<Actor>>>,
}

impl ActorController {
    pub fn new() -> Self {
        Self {
            id_to_actors: HashMap::new(),
            contracts_to_actors: HashMap::new(),
        }
    }

    pub fn add_actor(&mut self, actor: Actor) {
        let rc_actor = Rc::new(actor);
        self.id_to_actors
            .insert(rc_actor.block.block_id(), rc_actor.clone());
        for contract in rc_actor.contracts() {
            self.contracts_to_actors
                .entry(contract)
                .or_default()
                .push(rc_actor.clone());
        }
    }

    fn remove_actor_rc(&mut self, actor: &Rc<Actor>) {
        for contract in actor.contracts() {
            if let Some(actors) = self.contracts_to_actors.get_mut(&contract) {
                actors.retain(|a| !Rc::ptr_eq(a, actor));
                if actors.is_empty() {
                    self.contracts_to_actors.remove(&contract);
                }
            }
        }
        self.id_to_actors.remove(&actor.block.block_id());
    }

    pub fn get_actor_by_id(&self, id: u32) -> Option<Rc<Actor>> {
        self.id_to_actors.get(&id).cloned()
    }

    pub fn remove_actor_by_id(&mut self, id: u32) {
        if let Some(actor) = self.id_to_actors.remove(&id) {
            self.remove_actor_rc(&actor);
        }
    }
}

impl Default for ActorController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::block_macros::*;

    mod add_actor_indexes_by_id {
        use super::*;
        use ::block_traits::BlockSpec;

        make_defaults!(input, output, state, init_params);

        #[block(intents = ::intents::ZeroIntents, contract_deps = false)]
        pub struct TestBlock {
            pub block_id: u32,
        }

        impl BlockSpec for TestBlock {
            fn block_id(&self) -> u32 {
                self.block_id
            }

            fn new_from_init_params(_params: &InitParams) -> Self {
                Self { block_id: 0 }
            }

            fn init_state(&self) -> State {
                State
            }

            #[execute]
            fn execute(&self, _input: Input) -> Self::Intents {
                ::intents::ZeroIntents
            }
        }

        fn mk_actor(id: u32) -> Actor {
            let mut b = TestBlock::new_from_init_params(&InitParams);
            b.block_id = id;

            let reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};

            let reader =
                <InputKeys as channels::InputKeys<Input>>::reader(&input_keys, &reg).unwrap();
            let writer =
                <OutputKeys as channels::OutputKeys<Output>>::writer(&output_keys, &reg).unwrap();

            let block: Block = Block::new(b, reader, writer);
            Actor::new(block)
        }

        #[test]
        fn test() {
            let mut ctrl = ActorController::new();
            ctrl.add_actor(mk_actor(10));
            ctrl.add_actor(mk_actor(20));

            assert_eq!(ctrl.get_actor_by_id(10).unwrap().block.block_id(), 10);
            assert_eq!(ctrl.get_actor_by_id(20).unwrap().block.block_id(), 20);
        }
    }

    mod add_actor_indexes_by_contracts {
        use super::*;
        use ::block_traits::BlockSpec;
        use ::trade_types::Contract;

        make_defaults!(input, output, state);

        #[init_params]
        pub struct InitParams {
            pub contracts: Vec<Contract>,
        }

        #[block(intents = ::intents::ZeroIntents, contract_deps = true)]
        pub struct TestBlock {
            pub block_id: u32,
            pub contracts: Vec<Contract>,
        }

        impl BlockSpec for TestBlock {
            fn block_id(&self) -> u32 {
                self.block_id
            }

            fn new_from_init_params(params: &InitParams) -> Self {
                Self {
                    block_id: 0,
                    contracts: params.contracts.clone(),
                }
            }

            fn init_state(&self) -> State {
                State
            }

            #[execute]
            fn execute(&self, _input: Input) -> Self::Intents {
                ::intents::ZeroIntents
            }
        }

        fn c(name: &str) -> Contract {
            Contract::new(name)
        }

        fn mk_actor(id: u32, contracts: &[&str]) -> Actor {
            let reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};

            let reader =
                <InputKeys as channels::InputKeys<Input>>::reader(&input_keys, &reg).unwrap();
            let writer =
                <OutputKeys as channels::OutputKeys<Output>>::writer(&output_keys, &reg).unwrap();

            let params = InitParams {
                contracts: contracts.iter().map(|s| c(s)).collect(),
            };
            let mut b = TestBlock::new_from_init_params(&params);
            b.block_id = id;

            let block: Block = Block::new(b, reader, writer);
            Actor::new(block)
        }

        #[test]
        fn test() {
            let mut ctrl = ActorController::new();
            ctrl.add_actor(mk_actor(1, &["A", "B"]));
            ctrl.add_actor(mk_actor(2, &["B", "C"]));

            assert_eq!(ctrl.contracts_to_actors.get(&c("A")).unwrap().len(), 1);
            assert_eq!(ctrl.contracts_to_actors.get(&c("B")).unwrap().len(), 2);
            assert_eq!(ctrl.contracts_to_actors.get(&c("C")).unwrap().len(), 1);
            assert!(!ctrl.contracts_to_actors.contains_key(&c("D")));
        }
    }

    mod remove_actor_by_id_removes_from_id_and_contract_maps {
        use super::*;
        use ::block_traits::BlockSpec;
        use ::trade_types::Contract;

        make_defaults!(input, output, state);

        #[init_params]
        pub struct InitParams {
            pub contracts: Vec<Contract>,
        }

        #[block(intents = ::intents::ZeroIntents, contract_deps = true)]
        pub struct TestBlock {
            pub block_id: u32,
            pub contracts: Vec<Contract>,
        }

        impl BlockSpec for TestBlock {
            fn block_id(&self) -> u32 {
                self.block_id
            }

            fn new_from_init_params(params: &InitParams) -> Self {
                Self {
                    block_id: 0,
                    contracts: params.contracts.clone(),
                }
            }

            fn init_state(&self) -> State {
                State
            }

            #[execute]
            fn execute(&self, _input: Input) -> Self::Intents {
                ::intents::ZeroIntents
            }
        }

        fn c(name: &str) -> Contract {
            Contract::new(name)
        }

        fn mk_actor(id: u32, contracts: &[&str]) -> Actor {
            let reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};

            let reader =
                <InputKeys as channels::InputKeys<Input>>::reader(&input_keys, &reg).unwrap();
            let writer =
                <OutputKeys as channels::OutputKeys<Output>>::writer(&output_keys, &reg).unwrap();

            let params = InitParams {
                contracts: contracts.iter().map(|s| c(s)).collect(),
            };
            let mut b = TestBlock::new_from_init_params(&params);
            b.block_id = id;

            let block: Block = Block::new(b, reader, writer);
            Actor::new(block)
        }

        #[test]
        fn test() {
            let mut ctrl = ActorController::new();
            ctrl.add_actor(mk_actor(1, &["A", "B"]));
            ctrl.add_actor(mk_actor(2, &["B"]));

            ctrl.remove_actor_by_id(1);

            assert!(ctrl.get_actor_by_id(1).is_none());
            assert!(ctrl.get_actor_by_id(2).is_some());

            assert!(!ctrl.contracts_to_actors.contains_key(&c("A")));
            assert_eq!(ctrl.contracts_to_actors.get(&c("B")).unwrap().len(), 1);
            assert_eq!(
                ctrl.contracts_to_actors.get(&c("B")).unwrap()[0]
                    .block
                    .block_id(),
                2
            );
        }
    }
}
