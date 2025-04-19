use clap::{Arg, Command};
use std::time::Instant;
use std::io::Write;

mod generate_graphs;
mod k_wl;

fn main() {
    let matches = Command::new("Graph Generator")
        .version("1.0")
        .author("Hugo Hamon")
        .about("Generates non-isomorphic graphs of a given size")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("SIZE")
                .help("Sets the size of graphs to generate")
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    // Check if the size argument is provided
    if !matches.contains_id("size") {
        eprintln!("Error: The --size argument is required.");
        std::process::exit(1);
    }

    // Get the size from command line arguments
    let size = *matches.get_one::<usize>("size").unwrap();

    println!("Generating graphs of size: {}", size);

    // Measure the time taken to generate graphs
    let start_time = Instant::now();
    let graphs_dict = generate_graphs::generate_graphs(size);
    let duration = start_time.elapsed();

    println!(
        "Generated {} unique graph classes of size {}",
        graphs_dict.len(),
        size
    );
    println!("Time taken to generate graphs: {:?}", duration);

    // Save the graphs to files with the format "graphs_<size>/family_<index>.txt" with [(i, j), (i, )]
    for (i, (_, graphs)) in graphs_dict.iter().enumerate() {
        let filename = format!("graphs_{}/family_{}.txt", size, i);
        std::fs::create_dir_all(format!("graphs_{}", size)).unwrap();
        let mut file = std::fs::File::create(filename).unwrap();
        for graph in graphs {
            let edges: Vec<(usize, usize)> = graph
                .edge_indices()
                .map(|e| {
                    let (a, b) = graph.edge_endpoints(e).unwrap();
                    (a.index(), b.index())
                })
                .collect();
            
            // make the string representation of the graph
            let mut graph_str = format!(
                "[{}",
                edges
                    .iter()
                    .map(|(a, b)| format!("({}, {})", a, b))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            
            // Check if there is nodes with no edges and add them in the format (i, )
            let mut nodes_with_no_edges = Vec::new();
            for node in graph.node_indices() {
                if graph.edges(node).count() == 0 {
                    nodes_with_no_edges.push(node.index());
                }
            }
            if !nodes_with_no_edges.is_empty() {
                graph_str.push_str(&format!(
                    ",{}",
                    nodes_with_no_edges
                        .iter()
                        .map(|&a| format!("({}, )", a))
                        .collect::<Vec<String>>()
                        .join(", ")
                ));
            }
            graph_str.push_str("]");

            writeln!(file, "{}", graph_str).unwrap();
        }
    }
}
