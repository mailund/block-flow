use super::*;

/// Main trait for defining block behavior.
///
/// This trait extends `BlockSpecAssociatedTypes` with the core execution logic.
/// Blocks must implement `init_state` and `execute`, while the registry integration
/// methods have default implementations.
///
/// # Examples
///
/// ```rust
/// use block_macros::{block, init_params, input, output, state};
/// use block_traits::{BlockSpec, ExecutionContext};
/// use block_traits::intents::ZeroIntents;
///
/// #[input]
/// pub struct Input;
///
/// #[output]
/// pub struct Output {
///     pub is_after: bool,
/// }
///
/// #[state]
/// pub struct State;
///
/// #[init_params]
/// pub struct InitParams {
///     pub time: u64,
/// }
///
/// #[block]
/// pub struct AfterBlock {
///     pub block_id: u32,
///     time: u64,
/// }
///
/// impl BlockSpec for AfterBlock {
///     fn block_id(&self) -> u32 {
///         self.block_id
///     }
///
///     fn new_from_init_params(params: &InitParams) -> Self {
///         AfterBlock {
///             block_id: 0,
///             time: params.time,
///         }
///     }
///
///     fn init_state(&self) -> State {
///         State
///     }
///
///     fn execute(
///         &self,
///         context: &ExecutionContext,
///         _input: Input,
///         _state: &State,
///     ) -> Option<(Output, State, Self::Intents)> {
///         let is_after = context.time > self.time;
///         let output = Output { is_after };
///         Some((output, State, ZeroIntents::new()))
///     }
/// }
/// ```
pub trait BlockSpec: BlockSpecAssociatedTypes + ContractDeps {
    /// Return the ID of the block. Must be unique within an algorithm.
    fn block_id(&self) -> u32;

    /// Initialize the block's state.
    fn init_state(&self) -> Self::State;

    /// Create a new block instance from initialization parameters.
    fn new_from_init_params(params: &Self::InitParameters) -> Self;

    /// Execute the block's logic.
    ///
    /// When the block is type-erased into a `Block` the
    /// input and output will be handled by reading and writing to channels
    /// and the state will be managed by the wrapper.
    ///
    /// ```ignore
    ///
    ///   RefCell                              |                        ^
    ///                                        v                        |
    ///   execute: (ExecutionContext, Input, State) -> Option<(Output, State, Intents)>
    ///                                 ^                        |
    ///   Channels                      |                        v                        
    ///
    /// ```
    fn execute(
        &self,
        context: &ExecutionContext,
        input: Self::Input,
        state: &Self::State,
    ) -> Option<(Self::Output, Self::State, Self::Intents)>;
}

/// Default ContractDeps implementation for blocks without contract dependencies.
///
/// This should really only be used for testing purposes, but if you need a
/// a block to implement ContractDeps without any dependencies, you can use this marker.
/// You will need to implement the EmptyContractDepsTag for your block
/// ```ignore
/// impl ::block_traits::block_spec::EmptyContractDepsTag for YourBlock {}
/// ```
/// and then you get the ContractDeps for free.
pub trait EmptyContractDepsTag {}
impl<T> ContractDeps for T
where
    T: BlockSpec,
    T: EmptyContractDepsTag,
{
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        Vec::new()
    }
}
