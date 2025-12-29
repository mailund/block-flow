use super::*;

use super::actor::{ActorExecutionContext, ActorTrait};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trade_types::Contract;

#[derive(Clone)]
pub struct ActorHandle(Rc<RefCell<dyn ActorTrait>>);

impl ActorHandle {
    pub fn new(actor: impl ActorTrait + 'static) -> Self {
        Self(Rc::new(RefCell::new(actor)))
    }

    pub fn from_rc(rc: Rc<RefCell<dyn ActorTrait>>) -> Self {
        Self(rc)
    }

    pub fn as_rc(&self) -> Rc<RefCell<dyn ActorTrait>> {
        self.0.clone()
    }

    pub fn actor_id(&self) -> u32 {
        self.0.borrow().actor_id()
    }

    pub fn contracts(&self) -> Vec<Contract> {
        self.0.borrow().contracts()
    }

    pub fn tick(&self, context: &ActorExecutionContext) -> Option<()> {
        self.0.borrow_mut().tick(context)
    }

    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }
}

pub struct ActorController {
    time: u64, // mock time
    id_to_actors: HashMap<u32, ActorHandle>,
    contracts_to_actors: HashMap<Contract, Vec<ActorHandle>>,
}

impl ActorController {
    pub fn new() -> Self {
        Self {
            time: 0,
            id_to_actors: HashMap::new(),
            contracts_to_actors: HashMap::new(),
        }
    }

    pub fn add_actor(&mut self, actor: ActorHandle) {
        self.id_to_actors.insert(actor.actor_id(), actor.clone());
        for contract in actor.contracts() {
            self.contracts_to_actors
                .entry(contract)
                .or_default()
                .push(actor.clone());
        }
    }

    pub fn get_actor_by_id(&self, id: u32) -> Option<ActorHandle> {
        self.id_to_actors.get(&id).cloned()
    }

    pub fn remove_actor_by_id(&mut self, id: u32) {
        if let Some(actor) = self.id_to_actors.remove(&id) {
            self.remove_actor_rc_from_contract_tables(&actor);
        }
    }

    fn remove_actor_rc_from_contract_tables(&mut self, actor: &ActorHandle) {
        for contract in actor.contracts() {
            if let Some(actors) = self.contracts_to_actors.get_mut(&contract) {
                actors.retain(|a| !ActorHandle::ptr_eq(a, actor));
                if actors.is_empty() {
                    self.contracts_to_actors.remove(&contract);
                }
            }
        }
    }

    pub fn tick_delta(&mut self, Delta(contract): &Delta) {
        // Make a new execution context with the current state.
        let ctx = ActorExecutionContext::new(self.time);

        // Take the actor list out of the map so we can mutate `self` freely while iterating
        // and so we can remove actors that fail easily. Use .retain to keep only successful actors
        // (those that return None have failed and should be removed). For successful actors, emit their orders.
        // For failed actors, remove them from the id map and all contract lists.
        if let Some(mut actors) = self.contracts_to_actors.remove(contract) {
            actors.retain(|actor| {
                if actor.tick(&ctx).is_some() {
                    true // Successful, keep in list
                } else {
                    self.remove_actor_rc_from_contract_tables(actor);
                    false // Failed, remove from list
                }
            });

            // Put the survivors back (or drop the key if empty).
            if !actors.is_empty() {
                self.contracts_to_actors.insert(contract.clone(), actors);
            }
        }

        // Update state for next tick.
        self.time += 1;
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

        #[block(intents = block_traits::intents::ZeroIntents, contract_deps = false)]
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
                ::block_traits::intents::ZeroIntents
            }
        }

        fn mk_actor(id: u32) -> ActorHandle {
            use ::block_traits::BlockPackage;

            let mut reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};

            let package: BlockPackage<TestBlock> =
                BlockPackage::new(input_keys, output_keys, InitParams, None);
            let block = package.weave(&mut reg).unwrap();

            let actor: ActorHandle = ActorHandle::new(Actor::new(id, block));

            actor
        }

        #[test]
        fn test() {
            let mut ctrl = ActorController::new();
            ctrl.add_actor(mk_actor(10));
            ctrl.add_actor(mk_actor(20));

            assert_eq!((ctrl.get_actor_by_id(10).unwrap().actor_id()), 10);
            assert_eq!((ctrl.get_actor_by_id(20).unwrap().actor_id()), 20);
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

        #[block(intents = ::block_traits::intents::ZeroIntents, contract_deps = true)]
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
                ::block_traits::intents::ZeroIntents
            }
        }

        fn c(name: &str) -> Contract {
            Contract::new(name)
        }

        fn mk_actor(id: u32, contracts: &[&str]) -> ActorHandle {
            use ::block_traits::BlockPackage;

            let mut reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};
            let params = InitParams {
                contracts: contracts.iter().map(|s| c(s)).collect(),
            };
            let mut b = TestBlock::new_from_init_params(&params);
            b.block_id = id;

            let package = BlockPackage::<TestBlock>::new(input_keys, output_keys, params, None);
            let block = package.weave(&mut reg).unwrap();

            let actor: ActorHandle = ActorHandle::new(Actor::new(id, block));

            actor
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

        #[block(intents = ::block_traits::intents::ZeroIntents, contract_deps = true)]
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
                ::block_traits::intents::ZeroIntents
            }
        }

        fn c(name: &str) -> Contract {
            Contract::new(name)
        }

        fn mk_actor(id: u32, contracts: &[&str]) -> ActorHandle {
            use ::block_traits::BlockPackage;

            let mut reg = ::channels::ChannelRegistry::new();
            let input_keys = InputKeys {};
            let output_keys = OutputKeys {};
            let params = InitParams {
                contracts: contracts.iter().map(|s| c(s)).collect(),
            };
            let mut b = TestBlock::new_from_init_params(&params);
            b.block_id = id;

            let package: BlockPackage<TestBlock> =
                BlockPackage::new(input_keys, output_keys, params, None);
            let block = package.weave(&mut reg).unwrap();

            let actor: ActorHandle = ActorHandle::new(Actor::new(id, block));

            actor
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
                ctrl.contracts_to_actors.get(&c("B")).unwrap()[0].actor_id(),
                2
            );
        }
    }
}
