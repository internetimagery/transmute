use std::collections::HashMap;
use std::sync::Arc;

// python int
type Int = isize;

// Representing an edge between two nodes
// transforming from one thing to another
pub struct Edge {
    cost: Int,
    hash_in: Int,
    hash_out: Int,
}

pub struct Graph {
    map_in: HashMap<Int, Arc<Edge>>,
    map_out: HashMap<Int, Arc<Edge>>,
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
    pub fn add(
        &mut self,
        cost: Int,
        hash_in: Int,
        hash_out: Int,
    ) {
        let edge = Arc::new(Edge{
            cost, hash_in, hash_out
        });
        self.map_in.insert(hash_in, Arc::clone(&edge));
        self.map_out.insert(hash_out, Arc::clone(&edge));
    }

    // Search the graph to find what we want to find
    pub fn search(&self, hash_in: Int, hash_out: Int) {
        println!("SEARCHING!");
    }
}
