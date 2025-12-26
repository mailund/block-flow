use crate::{TopoOrdered, WeaveNode};
use channels::{errors::RegistryError, ChannelRegistry};
use std::collections::{HashMap, HashSet, VecDeque};

pub fn weave_nodes<W, T>(
    nodes: Vec<W>,
    registry: &mut ChannelRegistry,
) -> Result<TopoOrdered<T>, RegistryError>
where
    W: WeaveNode<T> + 'static,
    T: 'static,
{
    // Index nodes
    let n = nodes.len();

    // Collect inputs/outputs once (avoid recomputing, and keep ownership of Strings)
    let inputs: Vec<Vec<String>> = nodes.iter().map(|n| n.input_channels()).collect();
    let outputs: Vec<Vec<String>> = nodes.iter().map(|n| n.output_channels()).collect();

    // Map each output channel -> producer node index (error if duplicates)
    let mut producer_of: HashMap<String, usize> = HashMap::new();
    for (i, outs) in outputs.iter().enumerate() {
        for ch in outs {
            if producer_of.insert(ch.clone(), i).is_some() {
                return Err(RegistryError::DuplicateOutputKey(format!("'{ch}'")));
            }
        }
    }

    // Build graph edges producer -> consumer and indegrees
    let mut edges: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    let mut indegree: Vec<usize> = vec![0; n];

    for (consumer, ins) in inputs.iter().enumerate() {
        for ch in ins {
            if let Some(&producer) = producer_of.get(ch) {
                if producer != consumer && edges[producer].insert(consumer) {
                    indegree[consumer] += 1;
                }
            } else {
                // Allow external channels already present in registry, otherwise error.
                // If your registry uses a different API than `has`, change this.
                if !registry.has(ch) {
                    return Err(RegistryError::MissingProducer(format!(
                        "Missing producer for input channel '{ch}' (node index {consumer})"
                    )));
                }
            }
        }
    }

    // Kahn topological sort
    let mut q: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(i, &d)| (d == 0).then_some(i))
        .collect();

    let mut topo: Vec<usize> = Vec::with_capacity(n);
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

    // Weave in topo order
    let mut blocks = Vec::with_capacity(n);
    for idx in topo {
        blocks.push(nodes[idx].weave(registry)?);
    }

    Ok(TopoOrdered(blocks))
}
