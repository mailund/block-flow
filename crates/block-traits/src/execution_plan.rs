use crate::intents::SlotIntent;
use crate::{BlockTrait, ContractDeps, ExecutionContextTrait};
use ::channels::weave::TopoOrdered;

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
impl<C, BT> BlockTrait<C> for TopoOrdered<BT>
where
    C: ExecutionContextTrait,
    BT: BlockTrait<C>,
{
    // Collects and returns all SlotIntents produced by executing the blocks in the plan.
    // If any block fails to execute (returns None), the entire execution returns None.
    fn execute(&self, context: &C) -> Option<Vec<SlotIntent>> {
        // Execute each block in topological order,
        // flattening the resulting intents into a single vector.
        self.iter()
            // Get the optional output for each block
            .map(|block| block.execute(context))
            // Short-circuit if any block returned None
            .collect::<Option<Vec<Vec<SlotIntent>>>>()
            // Flatten the Vec<Vec<SlotIntent>> into Vec<SlotIntent>
            .map(|v| v.into_iter().flatten().collect())
    }
}
