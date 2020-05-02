use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// python int
pub type Int = isize;

// Representing an edge between two nodes
// transforming from one thing to another
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Edge {
    cost: Int,
    hash_in: Int,
    hash_out: Int,
}

// Our graph!
pub struct Graph {
    // TODO this needs to be a vector/set of Edges
    map_in: HashMap<Int, HashSet<Arc<Edge>>>,
    map_out: HashMap<Int, HashSet<Arc<Edge>>>,
}

impl Graph {
    // Create a new graph
    pub fn new() -> Self {
        Graph {
            map_in: HashMap::new(),
            map_out: HashMap::new(),
        }
    }

    // Add new edges to the graph
    pub fn add_edge(&mut self, cost: Int, hash_in: Int, hash_out: Int) {
        let edge = Arc::new(Edge {
            cost,
            hash_in,
            hash_out,
        });
        let edges_in = self.map_in.entry(hash_in).or_insert(HashSet::new());
        let edges_out = self.map_out.entry(hash_in).or_insert(HashSet::new());
        edges_in.insert(Arc::clone(&edge));
        edges_out.insert(Arc::clone(&edge));
    }

    // Search the graph to find what we want to find
    pub fn search(&self, hash_in: Int, hash_out: Int) {
        // TODO port across search logic
        println!("SEARCHING!");
    }
}
