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

struct Searcher<'a> {
    // what we have
    edges_in: &'a HashMap<Int, HashSet<Edge>>,
    edges_out: &'a HashMap<Int, HashSet<Edge>>,

    // what we want to find
    hash_in: Int,
    hash_out: Int,

    // our search queue
    queue_in: BinaryHeap<Reverse<State<'a>>>,
    queue_out: BinaryHeap<Reverse<State<'a>>>,

    // track where we have been
    visited_in: HashSet<&'a Edge>,
    visited_out: HashSet<&'a Edge>,
}

// Our graph!
pub struct Graph {
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

impl<'a> Searcher<'a> {
    fn new(
        hash_in: Int,
        hash_out: Int,
        edges_in: &'a HashMap<Int, HashSet<Edge>>,
        edges_out: &'a HashMap<Int, HashSet<Edge>>,
    ) -> Self {
        Searcher {
            edges_in,
            edges_out,
            hash_in,
            hash_out,
            queue_in: BinaryHeap::new(),
            queue_out: BinaryHeap::new(),
            visited_in: HashSet::new(),
            visited_out: HashSet::new(),
        }
    }

    fn search(&mut self) -> Option<Vec<Edge>> {
        self.set_queue_in();
        self.set_queue_out();

        loop {
            if !self.queue_in.is_empty() && self.queue_in.len() < self.queue_out.len() {
                if let Some(result) = self.search_forward() {
                    return Some(result);
                }
            } else if !self.queue_out.is_empty() {
                if let Some(result) = self.search_backward() {
                    return Some(result);
                }
            } else {
                break;
            }
        }
        None
    }

    fn search_forward(&mut self) -> Option<Vec<Edge>> {
        // next state
        let state = match self.queue_in.pop() {
            Some(Reverse(s)) => s,
            _ => return None,
        };

        // Check if we have reached our goal
        if state.edge.hash_out == self.hash_out {
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
        self.visited_in.insert(state.edge);

        // Search further into the graph!
        if let Some(edges) = self.edges_in.get(&state.edge.hash_out) {
            let state_rc = Rc::new(state);
            for edge in edges {
                if self.visited_in.contains(&edge) {
                    continue;
                }
                self.queue_in.push(Reverse(State::new(
                    edge.cost + state_rc.cost,
                    &edge,
                    Some(Rc::clone(&state_rc)),
                )))
            }
        }
        None
    }

    fn search_backward(&mut self) -> Option<Vec<Edge>> {
        let state = match self.queue_out.pop() {
            Some(Reverse(s)) => s,
            _ => return None,
        };

        // Check if we have reached our goal
        if state.edge.hash_in == self.hash_in {
            return Some(state.iter().map(|s| *s.edge).collect());
        }

        // Mark where we have been
        self.visited_out.insert(state.edge);

        // Search further into the graph!
        if let Some(edges) = self.edges_out.get(&state.edge.hash_in) {
            let state_rc = Rc::new(state);
            for edge in edges {
                if self.visited_in.contains(&edge) {
                    continue;
                }
                self.queue_out.push(Reverse(State::new(
                    edge.cost + state_rc.cost,
                    &edge,
                    Some(Rc::clone(&state_rc)),
                )))
            }
        }
        None
    }

    fn set_queue_in(&mut self) {
        if let Some(edges) = self.edges_in.get(&self.hash_in) {
            for edge in edges {
                self.queue_in
                    .push(Reverse(State::new(edge.cost, &edge, None)))
            }
        }
    }

    fn set_queue_out(&mut self) {
        if let Some(edges) = self.edges_out.get(&self.hash_out) {
            for edge in edges {
                self.queue_out
                    .push(Reverse(State::new(edge.cost, &edge, None)))
            }
        }
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
        let mut searcher = Searcher::new(hash_in, hash_out, &self.edges_in, &self.edges_out);
        searcher.search()
    }
}
