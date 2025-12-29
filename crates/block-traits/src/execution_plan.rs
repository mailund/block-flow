use crate::{ContractDeps, ExecuteTrait, ExecutionContextTrait, Intent};
use ::weave::TopoOrdered;

impl<CD> ContractDeps for TopoOrdered<CD>
where
    CD: ContractDeps,
{
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        let mut deps = Vec::new();
        for block in self.iter() {
            deps.extend(block.contract_deps());
        }
        deps
    }
}

/// Implementing the BlockTrait so execution plans can be used as composite blocks.
impl<C, X, I> ExecuteTrait<C, I> for TopoOrdered<X>
where
    C: ExecutionContextTrait,
    X: ExecuteTrait<C, I>,
    I: FnMut(&Intent),
{
    fn no_intents(&self) -> usize {
        // Sum the number of intents from each block in the plan
        self.iter().map(|block| block.no_intents()).sum()
    }
    // Collects and returns all SlotIntents produced by executing the blocks in the plan.
    // If any block fails to execute (returns None), the entire execution returns None.
    fn execute(&self, context: &C, intent_consumer: &mut I) -> Option<()> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        for block in self.iter() {
            block.execute(context, intent_consumer)?;
        }
        Some(())
    }
}
