use channels::{ChannelRegistry, RegistryError};
use serialization::Serializable;
use std::ops::Deref;

mod node_weaving;
pub use node_weaving::*;

pub trait NodePackage<E>: Serializable + Sized
where
    E: EmbeddedNode<Self>,
{
    // The NodePackage trait implements the weave node interface
    // explicitly so we don't have to explicitly implement WeaveNode
    // for every package type. The Weave node is then implemented
    // generically below.
    fn input_channels(&self) -> Vec<String>;
    fn output_channels(&self) -> Vec<String>;
    fn weave(&self, channels: &mut ChannelRegistry) -> Result<E, RegistryError>;
}

pub trait EmbeddedNode<P>: Sized + 'static
where
    P: NodePackage<Self>,
{
    fn extract_package(&self) -> P;
}

/// Something that can be weaved into a node in a graph
/// with edges connected through channels.
/// This trait allows for weaving of objects that do not have
/// a package/embedding pair.
pub trait WeaveNode<T> {
    fn input_channels(&self) -> Vec<String>;
    fn output_channels(&self) -> Vec<String>;
    fn weave(&self, channels: &mut ChannelRegistry) -> Result<T, RegistryError>;
}

impl<P, E> WeaveNode<E> for P
where
    P: NodePackage<E>,
    E: EmbeddedNode<P>,
{
    fn input_channels(&self) -> Vec<String> {
        NodePackage::<E>::input_channels(self)
    }

    fn output_channels(&self) -> Vec<String> {
        NodePackage::<E>::output_channels(self)
    }

    fn weave(&self, channels: &mut ChannelRegistry) -> Result<E, RegistryError> {
        NodePackage::<E>::weave(self, channels)
    }
}

/// Topologically ordered items for execution in a weave.
pub struct TopoOrdered<T>(pub Vec<T>);
impl<T> Deref for TopoOrdered<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
