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

    /// Input channels used for topological sorting before weaving.
    fn input_channels(&self) -> Vec<String>;

    /// Output channels used for topological sorting before weaving.
    fn output_channels(&self) -> Vec<String>;

    /// Register the channels used by this node package.
    ///
    /// This will usually be the output nodes, but it need not be. The output nodes
    /// are needed for topological sorting, but nodes can also register other channels
    /// that can be accessed by other nodes but need not be part of the topological sorting.
    fn register_channels(&self, channels: &mut ChannelRegistry) -> Result<(), RegistryError>;

    /// Weave the node into the given channel registry.
    fn weave(&self, channels: &mut ChannelRegistry) -> Result<E, RegistryError>;
}

pub trait EmbeddedNode<P>: Sized + 'static
where
    P: NodePackage<Self>,
{
    /// Extract the package used to weave this node.
    ///
    /// This is mainly useful for serialization of the node and allows an embedding
    /// to hold updated data that should be part of the serialized package. The idea
    /// is that weave and extract are inverses, but any state hold by the embedding
    /// will break that invariant (and intentionally so).
    fn extract_package(&self) -> P;
}

/// Something that can be weaved into a node in a graph
/// with edges connected through channels.
/// This trait allows for weaving of objects that do not have
/// a package/embedding pair.
pub trait WeaveNode<E> {
    /// Input channels used for topological sorting before weaving.
    fn input_channels(&self) -> Vec<String>;

    /// Output channels used for topological sorting before weaving.
    fn output_channels(&self) -> Vec<String>;

    /// Register the channels used by this node package.
    ///
    /// This will usually be the output nodes, but it need not be. The output nodes
    /// are needed for topological sorting, but nodes can also register other channels
    /// that can be accessed by other nodes but need not be part of the topological sorting.
    fn register_channels(&self, channels: &mut ChannelRegistry) -> Result<(), RegistryError>;

    /// Weave the node into the given channel registry.
    fn weave(&self, channels: &mut ChannelRegistry) -> Result<E, RegistryError>;
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
    fn register_channels(&self, channels: &mut ChannelRegistry) -> Result<(), RegistryError> {
        NodePackage::<E>::register_channels(self, channels)
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
