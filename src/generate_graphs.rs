use petgraph::algo::is_isomorphic as petgraph_is_isomorphic;
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

use crate::k_wl;

/// Generate all unique graphs up to a maximum size and group isomorphic graphs together.
pub fn generate_graphs(max_size: usize) -> HashMap<String, Vec<UnGraph<(), ()>>> {
    if max_size < 1 {
        panic!("size must be greater than or equal to 1");
    }

    // Make the starting graph with one node
    let mut starting_graph = UnGraph::<(), ()>::new_undirected();
    starting_graph.add_node(());

    // Dictionary to store unique graphs by their hash
    let mut hashes: HashMap<String, Vec<UnGraph<(), ()>>> = HashMap::new();

    // Add a graph to the hashes dictionary if it's unique
    fn add_element_to_hashes(
        element: &UnGraph<(), ()>,
        hashes: &mut HashMap<String, Vec<UnGraph<(), ()>>>,
    ) -> bool {
        let graph_hash_1wl = k_wl::k_wl(element, 1, -1);

        if !hashes.contains_key(&graph_hash_1wl) {
            hashes.insert(graph_hash_1wl.clone(), Vec::new());
        }

        let mut to_add = true;
        if let Some(graphs) = hashes.get(&graph_hash_1wl) {
            for g in graphs {
                if petgraph_is_isomorphic(element, g) {
                    to_add = false;
                    break;
                }
            }
        }

        if to_add {
            if let Some(graphs) = hashes.get_mut(&graph_hash_1wl) {
                graphs.push(element.clone());
            }
        }
        to_add
    }

    // Recursively generate all possible graphs
    fn recursive_generate(
        element: UnGraph<(), ()>,
        max_size: usize,
        hashes: &mut HashMap<String, Vec<UnGraph<(), ()>>>,
    ) {
        let mut new_starting_graph = element.clone();
        let new_node = new_starting_graph.add_node(());

        if new_starting_graph.node_count() > max_size {
            return;
        }

        // Generate all possible combinations of graph that connect the new node to the existing nodes
        let edges: Vec<(NodeIndex, NodeIndex)> = new_starting_graph
            .node_indices()
            .filter(|&i| i != new_node)
            .map(|i| (new_node, i))
            .collect();

        // Iterate through all possible edge combinations (2^n possibilities)
        let num_combinations = 1 << edges.len();
        for i in 0..num_combinations {
            let mut new_graph = new_starting_graph.clone();

            for j in 0..edges.len() {
                if (i >> j) & 1 == 1 {
                    let (a, b) = edges[j];
                    new_graph.add_edge(a, b, ());
                }
            }

            if add_element_to_hashes(&new_graph, hashes) {
                recursive_generate(new_graph, max_size, hashes);
            }
        }
    }

    // Start the recursive process
    add_element_to_hashes(&starting_graph, &mut hashes);
    recursive_generate(starting_graph, max_size, &mut hashes);

    // Print the number of unique graphs found
    println!("Found {} unique graphs", hashes.len());

    // Keep only the graphs that are of size max_size
    let hash_keys: Vec<String> = hashes.keys().cloned().collect();
    for graph_hash in hash_keys {
        if let Some(graphs) = hashes.get(&graph_hash) {
            if graphs.len() <= 1 {
                hashes.remove(&graph_hash);
                continue;
            }

            let filtered_graphs: Vec<UnGraph<(), ()>> = graphs
                .iter()
                .filter(|g| g.node_count() == max_size)
                .cloned()
                .collect();

            if filtered_graphs.is_empty() {
                hashes.remove(&graph_hash);
            } else {
                hashes.insert(graph_hash, filtered_graphs);
            }
        }
    }

    println!("Found {} unique graphs of size {}", hashes.len(), max_size);
    hashes
}
