use super::errors;
use super::ChannelRegistry;

/// Trait for readers that can read values of type T
pub trait Reader<T> {
    fn read(&self) -> T;
}

/// Trait for keys that work along channels. Used for mapping
/// input/output keys to their channel names.
pub trait ChannelKeys: Clone + std::fmt::Debug {
    fn channel_names(&self) -> Vec<String>;
}

/// Trait for keys that can create readers
pub trait InputKeys<T>: ChannelKeys {
    type ReaderType: Reader<T>;
    fn reader(&self, registry: &ChannelRegistry)
        -> Result<Self::ReaderType, errors::RegistryError>;
}

/// Trait for writers that can write values of type T
pub trait Writer<T> {
    fn write(&self, output: &T);
}

/// Trait for keys that can create writers
pub trait OutputKeys<T>: ChannelKeys {
    type WriterType: Writer<T>;
    fn writer(&self, registry: &ChannelRegistry)
        -> Result<Self::WriterType, errors::RegistryError>;
    fn register(&self, registry: &mut ChannelRegistry);
}
