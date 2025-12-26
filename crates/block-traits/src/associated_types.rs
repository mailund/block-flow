use channels::{InputKeys, OutputKeys};
use serialization::Serializable;

/// Trait for block input data types.
///
/// This trait defines the associated types needed for a block's input.
/// The `Keys` type must implement `InputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use block_traits::*;
/// use ::channels::*;
/// use ::serde::{Serialize, Deserialize};
/// use ::serialization::structs::Serializable;
///
/// // Define a simple input type
/// #[derive(Clone)]
/// struct SimpleInput {
///     value: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize, Debug, Clone)]
/// struct SimpleInputKeys;
///
/// impl Serializable for SimpleInputKeys {}
///
/// impl ChannelKeys for SimpleInputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// // Mock reader for testing
/// struct MockReader<T> { data: T }
/// impl<T: Clone> Reader<T> for MockReader<T> {
///     fn read(&self) -> T { self.data.clone() }
/// }
///
/// impl InputKeys<SimpleInput> for SimpleInputKeys {
///     type ReaderType = MockReader<SimpleInput>;
///
///     fn reader(&self, _registry: &ChannelRegistry) -> Result<Self::ReaderType, RegistryError> {
///         Ok(MockReader { data: SimpleInput { value: 42 } })
///     }
/// }
///
/// impl BlockInput for SimpleInput {
///     type Keys = SimpleInputKeys;
/// }
/// ```
pub trait BlockInput: Sized {
    type Keys: InputKeys<Self> + Serializable;
}

/// Trait for block output data types.
///
/// This trait defines the associated types needed for a block's output.
/// The `Keys` type must implement `OutputKeys` to provide registry integration.
///
/// # Examples
///
/// ```rust
/// use ::block_traits::*;
/// use ::channels::*;
/// use ::serde::{Serialize, Deserialize};
/// use ::serialization::structs::Serializable;
///
/// // Define a simple output type
/// #[derive(Clone)]
/// struct SimpleOutput {
///     result: i32,
/// }
///
/// // Implement the required Keys type (this would typically be auto-generated)
/// #[derive(Serialize, Deserialize, Debug, Clone)]
/// struct SimpleOutputKeys;
///
/// impl Serializable for SimpleOutputKeys {}
///
/// impl ChannelKeys for SimpleOutputKeys {
///     fn channel_names(&self) -> Vec<String> { vec![] }
/// }
///
/// // Mock writer for testing
/// struct MockWriter<T> {
///     written: std::cell::RefCell<Option<T>>
/// }
///
/// impl<T: Clone> Writer<T> for MockWriter<T> {
///     fn write(&self, data: &T) {
///         *self.written.borrow_mut() = Some(data.clone());
///     }
/// }
///
/// impl OutputKeys<SimpleOutput> for SimpleOutputKeys {
///     type WriterType = MockWriter<SimpleOutput>;
///
///     fn writer(&self, _registry: &ChannelRegistry) -> Result<Self::WriterType, RegistryError> {
///         Ok(MockWriter { written: std::cell::RefCell::new(None) })
///     }
///
///     fn register(&self, _registry: &mut ChannelRegistry) -> Result<(), RegistryError> {
///         // Registration logic would go here
///        Ok(())
///     }
/// }
///
/// impl BlockOutput for SimpleOutput {
///     type Keys = SimpleOutputKeys;
/// }
/// ```
pub trait BlockOutput: Sized {
    type Keys: OutputKeys<Self> + Serializable;
}

/// Trait for retrieving contract dependencies of a block.
pub trait ContractDeps {
    fn contract_deps(&self) -> Vec<::trade_types::Contract> {
        Vec::new()
    }
}

pub trait BlockSpecAssociatedTypes {
    type Input: BlockInput;
    type Output: BlockOutput;
    type State: Clone + Serializable;
    type InitParameters: Clone + ContractDeps + Serializable;
    type Intents: crate::intents::BlockIntents;
}

/// Type aliases for input reader and output writer for a given block spec.
/// Just to make the code below more readable.
pub mod block_keys {
    use super::*;

    pub type In<B> = <B as BlockSpecAssociatedTypes>::Input;
    pub type Out<B> = <B as BlockSpecAssociatedTypes>::Output;

    pub type InKeys<B> = <In<B> as BlockInput>::Keys;
    pub type OutKeys<B> = <Out<B> as BlockOutput>::Keys;

    pub type InReader<B> = <InKeys<B> as InputKeys<In<B>>>::ReaderType;
    pub type OutWriter<B> = <OutKeys<B> as OutputKeys<Out<B>>>::WriterType;
}
