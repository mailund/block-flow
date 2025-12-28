use crate::{TopoOrdered, WeaveNode};
use channels::{errors::RegistryError, ChannelRegistry};
use std::collections::{HashMap, HashSet, VecDeque};

/// Topologically sort the nodes and ensuring that all input channels have a producer,
/// that only one node produces each output channel, and that there are no cycles.
/// Once this is guaranteed, weave the nodes in topological order.
pub fn weave_nodes<W, T>(
    nodes: &[W],
    registry: &mut ChannelRegistry,
) -> Result<TopoOrdered<T>, RegistryError>
where
    W: WeaveNode<T> + 'static,
    T: 'static,
{
    // First, register all channels used by the nodes. Usually, this will be just the output
    // channels, but nodes may register other channels as well that can be used by other
    // nodes but that will not affect the topological ordering.
    register_all_channels(nodes, registry)?;

    // Compute the topological ordering of the nodes based on their input/output channels.
    let producer_of = producer_map(nodes);
    let edges = build_edges(nodes, registry, &producer_of)?;
    let topo = topo_order_or_cycle(&edges)?;

    // Finally, weave the nodes in topological order.
    let mut out = Vec::with_capacity(nodes.len());
    for idx in topo {
        out.push(nodes[idx].weave(registry)?);
    }
    Ok(TopoOrdered(out))
}

/// Register all channels of all nodes in the registry.
/// This might be more than the output channels, but the output channels must be registered
/// as we later consider it an error if an input channel has no producer in the registry.
/// If multiple nodes produce the same output channel, this will be an error in the
/// registration and will propagate up.
fn register_all_channels<W, T>(
    nodes: &[W],
    registry: &mut ChannelRegistry,
) -> Result<(), RegistryError>
where
    W: WeaveNode<T>,
    T: 'static,
{
    for n in nodes {
        n.register_channels(registry)?;
    }
    Ok(())
}

/// Map each output channel to the index of the node that produces it.
/// This is used in the topological sort.
fn producer_map<W, T>(nodes: &[W]) -> HashMap<String, usize>
where
    W: WeaveNode<T>,
    T: 'static,
{
    let mut producer_of = HashMap::<String, usize>::new();
    for (i, node) in nodes.iter().enumerate() {
        for ch in node.output_channels() {
            producer_of.insert(ch, i);
        }
    }
    producer_of
}

/// Build the edges of the graph representing dependencies between nodes.
/// An edge from node A to node B means that B depends on A; in this case that
/// B has an input channel produced by A.
///
/// This only takes input/output channels into account. It is valid for nodes
/// to register other channels, and for other nodes to read from them, but this
/// will not affect the topological ordering.
fn build_edges<W, T>(
    nodes: &[W],
    registry: &ChannelRegistry,
    producer_of: &HashMap<String, usize>,
) -> Result<Vec<HashSet<usize>>, RegistryError>
where
    W: WeaveNode<T>,
    T: 'static,
{
    let n = nodes.len();
    let mut edges: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    let mut indegree: Vec<usize> = vec![0; n];

    for (consumer, node) in nodes.iter().enumerate() {
        for ch in node.input_channels() {
            if let Some(&producer) = producer_of.get(&ch) {
                if producer != consumer && edges[producer].insert(consumer) {
                    indegree[consumer] += 1;
                }
            } else if !registry.has(&ch) {
                return Err(RegistryError::MissingProducer(format!(
                    "Missing producer for input channel '{ch}' (node index {consumer})"
                )));
            }
        }
    }

    Ok(edges)
}

/// Perform a topological sort on the given edges. If a cycle is detected,
/// an error is returned with the indices of the nodes involved in the cycle.
/// Otherwise, we return a vector of node indices in topological order.
fn topo_order_or_cycle(edges: &[HashSet<usize>]) -> Result<Vec<usize>, RegistryError> {
    let n = edges.len();

    let mut indegree = vec![0usize; n];
    for neighbors in edges.iter() {
        for &v in neighbors {
            indegree[v] += 1;
        }
    }

    let mut q: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(i, &d)| (d == 0).then_some(i))
        .collect();

    let mut topo = Vec::with_capacity(n);
    while let Some(u) = q.pop_front() {
        topo.push(u);
        for &v in edges[u].iter() {
            indegree[v] -= 1;
            if indegree[v] == 0 {
                q.push_back(v);
            }
        }
    }

    if topo.len() != n {
        let cyclic: Vec<usize> = indegree
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| (d > 0).then_some(i))
            .collect();
        return Err(RegistryError::CycleDetected(format!("{cyclic:?}")));
    }

    Ok(topo)
}
