use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

// python int
pub type Int = isize;

// Representing an edge between two nodes
// transforming from one thing to another
#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct Edge {
    cost: Int,
    hash_in: Int,
    hash_out: Int,
    hash_func: Int,
}

#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct State {
    cost: Int,
    actual_cost: Int,
    edge: Arc<Edge>,
    parent: Option<Rc<State>>,
}

struct StateIter<'a> {
    node: Option<&'a State>,
}

// Our graph!
pub struct Graph {
    // TODO this needs to be a vector/set of Edges
    edges_in: HashMap<Int, HashSet<Arc<Edge>>>,
    edges_out: HashMap<Int, HashSet<Arc<Edge>>>,
}

impl State {
    fn new(cost: Int, edge: Arc<Edge>, parent: Option<Rc<State>>) -> Self {
        let actual_cost = edge.cost
            + match &parent {
                Some(p) => p.cost,
                None => 0,
            };
        Self {
            cost,
            actual_cost,
            edge,
            parent,
        }
    }
    fn iter(&self) -> StateIter {
        StateIter { node: Some(self) }
    }
}

impl<'a> Iterator for StateIter<'a> {
    type Item = &'a State;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = match &node.parent {
                Some(parent) => Some(parent),
                None => None,
            };
            return Some(node);
        }
        None
    }
}

impl Graph {
    // Create a new graph
    pub fn new() -> Self {
        Graph {
            edges_in: HashMap::new(),
            edges_out: HashMap::new(),
        }
    }

    // Add new edges to the graph
    pub fn add_edge(&mut self, cost: Int, hash_in: Int, hash_out: Int, hash_func: Int) {
        let edge = Arc::new(Edge {
            cost,
            hash_in,
            hash_out,
            hash_func,
        });
        let edges_in = self.edges_in.entry(hash_in).or_insert(HashSet::new());
        let edges_out = self.edges_out.entry(hash_out).or_insert(HashSet::new());
        edges_in.insert(Arc::clone(&edge));
        edges_out.insert(edge);
    }

    // Search the graph to find what we want to find
    pub fn search(&self, hash_in: Int, hash_out: Int) -> Option<Vec<Arc<Edge>>> {
        // Get our starting points!
        let mut queue_in: BinaryHeap<_> = match self.edges_in.get(&hash_in) {
            Some(edges) => edges
                .iter()
                .map(|e| Reverse(State::new(e.cost, Arc::clone(&e), None)))
                .collect(),
            None => BinaryHeap::new(),
        };
        let mut queue_out: BinaryHeap<_> = match self.edges_out.get(&hash_out) {
            Some(edges) => edges
                .iter()
                .map(|e| Reverse(State::new(e.cost, Arc::clone(e), None)))
                .collect(),
            None => BinaryHeap::new(),
        };

        // Track where we have been
        let mut visited_in = HashSet::new();
        let mut visited_out = HashSet::new();

        // Search our graph!
        loop {
            if !queue_in.is_empty() && queue_in.len() < queue_out.len() {
                // //////////////// //
                // Search forwards! //
                // //////////////// //
                let state = match queue_in.pop() {
                    Some(Reverse(s)) => s,
                    _ => continue,
                };

                // Check if we have reached our goal
                if state.edge.hash_out == hash_out {
                    println!("We did it!");
                    return Some(Vec::new());
                }

                // Mark where we have been
                visited_in.insert(Arc::clone(&state.edge));

                // Search further into the graph!
                if let Some(edges) = self.edges_in.get(&state.edge.hash_out) {
                    let state_rc = Rc::new(state);
                    for edge in edges {
                        if visited_in.contains(edge.as_ref()) {
                            continue;
                        }
                        queue_in.push(Reverse(State::new(
                            1,
                            Arc::clone(edge),
                            Some(Rc::clone(&state_rc)),
                        )))
                    }
                }
            } else if !queue_out.is_empty() {
                // ///////////////// //
                // Search backwards! //
                // ///////////////// //
                let state = match queue_out.pop() {
                    Some(Reverse(s)) => s,
                    _ => continue,
                };

                // Check if we have reached our goal
                if state.edge.hash_in == hash_in {
                    println!("We did it!");
                    return Some(state.iter().map(|s| Arc::clone(&s.edge)).collect());
                }

                // Mark where we have been
                visited_out.insert(Arc::clone(&state.edge));

                // Search further into the graph!
                if let Some(edges) = self.edges_out.get(&state.edge.hash_in) {
                    let state_rc = Rc::new(state);
                    for edge in edges {
                        if visited_in.contains(edge.as_ref()) {
                            continue;
                        }
                        queue_out.push(Reverse(State::new(
                            1,
                            Arc::clone(edge),
                            Some(Rc::clone(&state_rc)),
                        )))
                    }
                }
            } else {
                break;
            }
        }
        println!("Queue is empty");
        None
    }
}
