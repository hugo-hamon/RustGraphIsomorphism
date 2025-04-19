use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use petgraph::graph::{NodeIndex, UnGraph};
use sha2::{Digest, Sha256};

/// Compute the atomic type of a k-tuple in the graph.
/// The atomic type is a Vec of booleans indicating the presence of edges between the nodes in the k-tuple.
#[inline]
fn atomic_type(k_tuple: &[NodeIndex], graph: &UnGraph<(), ()>) -> Vec<u8> {
    let k = k_tuple.len();
    let mut signature = Vec::with_capacity(k * (k - 1) / 2);

    for i in 0..k {
        for j in (i + 1)..k {
            signature.push(graph.contains_edge(k_tuple[i], k_tuple[j]) as u8);
        }
    }

    signature
}

/// Get the neighbors of a k-tuple for a given index in the graph.
#[inline]
fn get_neighbors(
    k_tuple: &[NodeIndex],
    index: usize,
    graph_nodes: &[NodeIndex],
) -> Vec<Vec<NodeIndex>> {
    graph_nodes
        .iter()
        .map(|&w| {
            let mut new_tuple = k_tuple.to_vec();
            new_tuple[index] = w;
            new_tuple
        })
        .collect()
}

/// Calculate a deterministic hash of an object.
#[inline]
fn deterministic_hash<T: Hash>(obj: T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    obj.hash(&mut hasher);
    let hash_value = hasher.finish();

    let mut sha = Sha256::new();
    sha.update(hash_value.to_le_bytes());
    format!("{:x}", sha.finalize())
}

/// k-WL algorithm. If k_wl(G1) != k_wl(G2) then G1 and G2 are not isomorphic.
/// If k_wl(G1) == k_wl(G2) then G1 and G2 may be isomorphic but not necessarily.
pub fn k_wl(graph: &UnGraph<(), ()>, k: usize, iterations: isize) -> String {
    if k < 1 {
        panic!("k must be greater than or equal to 1");
    }
    if iterations != -1 && iterations < 1 {
        panic!("iterations must be -1 or greater than or equal to 1");
    }

    let iterations = if iterations == -1 {
        graph.node_count() as isize
    } else {
        iterations
    };

    if k == 1 {
        return weisfeiler_lehman_graph_hash(graph, iterations as usize);
    }

    let nodes: Vec<NodeIndex> = graph.node_indices().collect();
    let mut k_tuples = Vec::new();

    // Generate all k-tuples
    fn generate_tuples(
        nodes: &[NodeIndex],
        k: usize,
        current: &mut Vec<NodeIndex>,
        result: &mut Vec<Vec<NodeIndex>>,
    ) {
        if current.len() == k {
            result.push(current.clone());
            return;
        }

        for &node in nodes {
            current.push(node);
            generate_tuples(nodes, k, current, result);
            current.pop();
        }
    }

    generate_tuples(&nodes, k, &mut Vec::new(), &mut k_tuples);

    // Initialize colors based on atomic types
    let mut colors: HashMap<Vec<NodeIndex>, usize> = HashMap::new();
    let mut color_classes: HashMap<Vec<u8>, Vec<Vec<NodeIndex>>> = HashMap::new();

    for k_tuple in &k_tuples {
        let boolean_signature = atomic_type(k_tuple, graph);
        color_classes
            .entry(boolean_signature)
            .or_default()
            .push(k_tuple.clone());
    }

    let mut next_color = 0;
    let mut sorted_keys: Vec<Vec<u8>> = color_classes.keys().cloned().collect();
    sorted_keys.sort();

    for signature in sorted_keys {
        if let Some(tuples) = color_classes.get(&signature) {
            for k_tuple in tuples {
                colors.insert(k_tuple.clone(), next_color);
            }
            next_color += 1;
        }
    }

    for _ in 0..iterations {
        let mut new_color_classes: HashMap<Vec<usize>, Vec<Vec<NodeIndex>>> = HashMap::new();

        for k_tuple in &k_tuples {
            let mut neighbor_colors = Vec::new();

            for i in 0..k {
                let neighbors = get_neighbors(k_tuple, i, &nodes);
                let mut multiset: Vec<usize> = neighbors
                    .iter()
                    .map(|neighbor| *colors.get(neighbor).unwrap_or(&0))
                    .collect();
                multiset.sort();
                neighbor_colors.push(multiset);
            }

            let mut signature = vec![*colors.get(k_tuple).unwrap_or(&0)];
            for nc in neighbor_colors {
                signature.extend(nc);
            }

            new_color_classes
                .entry(signature)
                .or_default()
                .push(k_tuple.clone());
        }

        let mut new_colors: HashMap<Vec<NodeIndex>, usize> = HashMap::new();
        let mut next_color = 0;

        let mut sorted_signatures: Vec<Vec<usize>> = new_color_classes.keys().cloned().collect();
        sorted_signatures.sort();

        for signature in sorted_signatures {
            if let Some(tuples) = new_color_classes.get(&signature) {
                for k_tuple in tuples {
                    new_colors.insert(k_tuple.clone(), next_color);
                }
                next_color += 1;
            }
        }

        if new_colors == colors {
            break;
        }

        colors = new_colors;
    }

    // Final multiset
    let mut final_multiset: Vec<usize> = colors.values().cloned().collect();
    final_multiset.sort();

    deterministic_hash(final_multiset)
}

/// Implementation of the 1-WL algorithm for graph hashing
fn weisfeiler_lehman_graph_hash(graph: &UnGraph<(), ()>, iterations: usize) -> String {
    let node_count = graph.node_count();
    let mut node_labels: HashMap<NodeIndex, String> = HashMap::with_capacity(node_count);
    
    // Pre-compute degrees
    let mut degrees = HashMap::with_capacity(node_count);
    for node in graph.node_indices() {
        let degree = graph.neighbors(node).count();
        degrees.insert(node, degree);
        node_labels.insert(node, degree.to_string());
    }

    let mut subgraph_hash_counts = Vec::new();
    
    for _ in 0..iterations {
        // Apply neighborhood aggregation for each node
        let mut new_labels = HashMap::with_capacity(node_count);
        
        for node in graph.node_indices() {
            // Collect and sort neighbor labels more efficiently
            let mut neighbor_labels: Vec<_> = graph.neighbors(node)
                .filter_map(|neighbor| node_labels.get(&neighbor).cloned())
                .collect();
            neighbor_labels.sort_unstable();
            
            // Create new label by combining current label with sorted neighbor labels
            let label = node_labels.get(&node).unwrap_or(&String::new()).clone() + 
                        &neighbor_labels.join("");
            
            // Hash the label
            let hashed_label = deterministic_hash(label);
            new_labels.insert(node, hashed_label);
        }
        
        // Update node labels
        node_labels = new_labels;
        
        // Count label occurrences
        let mut counter = HashMap::new();
        for label in node_labels.values() {
            *counter.entry(label.clone()).or_insert(0) += 1;
        }
        
        // Sort counter items by label
        let mut sorted_items: Vec<(String, usize)> = counter.into_iter().collect();
        sorted_items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        
        subgraph_hash_counts.extend(sorted_items);
    }
    
    // Hash the final counter
    deterministic_hash(subgraph_hash_counts)
}