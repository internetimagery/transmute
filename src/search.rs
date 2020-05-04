use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::rc::Rc;

// python int
pub type Int = isize;

// Representing an edge between two nodes
// transforming from one thing to another
#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd, Clone, Copy)]
pub struct Edge {
    cost: Int,
    hash_in: Int,
    hash_out: Int,
    pub hash_func: Int,
}

#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct State<'a> {
    cost: Int,
    actual_cost: Int,
    edge: &'a Edge,
    parent: Option<Rc<State<'a>>>,
}

struct StateIter<'a> {
    node: Option<&'a State<'a>>,
}

// Our graph!
pub struct Graph {
    // TODO this needs to be a vector/set of Edges
    edges_in: HashMap<Int, HashSet<Edge>>,
    edges_out: HashMap<Int, HashSet<Edge>>,
}

impl<'a> State<'a> {
    fn new(cost: Int, edge: &'a Edge, parent: Option<Rc<State<'a>>>) -> Self {
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
    type Item = &'a State<'a>;

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
        let edge = Edge {
            cost,
            hash_in,
            hash_out,
            hash_func,
        };
        let edges_in = self.edges_in.entry(hash_in).or_insert(HashSet::new());
        let edges_out = self.edges_out.entry(hash_out).or_insert(HashSet::new());
        edges_in.insert(edge);
        edges_out.insert(edge);
    }

    // Search the graph to find what we want to find
    pub fn search(&self, hash_in: Int, hash_out: Int) -> Option<Vec<Edge>> {
        // Get our starting points!
        let mut queue_in: BinaryHeap<_> = match self.edges_in.get(&hash_in) {
            Some(edges) => edges
                .iter()
                .map(|e| Reverse(State::new(e.cost, &e, None)))
                .collect(),
            None => BinaryHeap::new(),
        };
        let mut queue_out: BinaryHeap<_> = match self.edges_out.get(&hash_out) {
            Some(edges) => edges
                .iter()
                .map(|e| Reverse(State::new(e.cost, &e, None)))
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
                    return Some(
                        state
                            .iter()
                            .map(|s| *s.edge)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect(),
                    );
                }

                // Mark where we have been
                visited_in.insert(state.edge);

                // Search further into the graph!
                if let Some(edges) = self.edges_in.get(&state.edge.hash_out) {
                    let parent_cost = match &state.parent {
                        Some(parent) => parent.cost,
                        None => 0,
                    };
                    let state_rc = Rc::new(state);
                    for edge in edges {
                        if visited_in.contains(&edge) {
                            continue;
                        }
                        queue_in.push(Reverse(State::new(
                            edge.cost + parent_cost,
                            &edge,
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
                    return Some(state.iter().map(|s| *s.edge).collect());
                }

                // Mark where we have been
                visited_out.insert(state.edge);

                // Search further into the graph!
                if let Some(edges) = self.edges_out.get(&state.edge.hash_in) {
                    let parent_cost = match &state.parent {
                        Some(parent) => parent.cost,
                        None => 0,
                    };
                    let state_rc = Rc::new(state);
                    for edge in edges {
                        if visited_in.contains(&edge) {
                            continue;
                        }
                        queue_out.push(Reverse(State::new(
                            edge.cost + parent_cost,
                            &edge,
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
