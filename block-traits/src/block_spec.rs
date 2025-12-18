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
/// use intents::ZeroIntents;
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
///     ) -> (Output, State, Self::Intents) {
///         let is_after = context.time > self.time;
///         let output = Output { is_after };
///         (output, State, ZeroIntents::new())
///     }
/// }
/// ```
pub trait BlockSpec: BlockSpecAssociatedTypes {
    fn block_id(&self) -> u32;

    fn init_state(&self) -> Self::State;

    fn new_from_init_params(params: &Self::InitParameters) -> Self;

    fn execute(
        &self,
        context: &ExecutionContext,
        input: Self::Input,
        state: &Self::State,
    ) -> (Self::Output, Self::State, Self::Intents);
}
