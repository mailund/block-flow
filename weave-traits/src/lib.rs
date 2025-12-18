use channels::{ChannelRegistry, RegistryError};
use std::ops::Deref;

/// Something that can be weaved into a node in a graph
/// with edges connected through channels.
pub trait WeaveNode<T> {
    fn input_channels(&self) -> Vec<String>;
    fn output_channels(&self) -> Vec<String>;
    fn weave(&self, channels: &mut ChannelRegistry) -> Result<T, RegistryError>;
}

/// Topologically ordered items for execution in a weave.
pub struct TopoOrdered<T>(pub Vec<T>);
impl<T> Deref for TopoOrdered<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
