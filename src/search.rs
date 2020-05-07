use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

// python int
pub type Int = isize;
pub type Variations = BTreeSet<Int>;

// Representing an edge between two nodes
// transforming from one thing to another
#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct Edge {
    cost: Int,
    hash_in: Int,
    hash_out: Int,
    pub hash_func: Int,
    hash_var_in: Variations,
    hash_var_out: Variations,
}

#[derive(Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct State<'a> {
    cost: Int,
    actual_cost: Int,
    edge: &'a Arc<Edge>,
    parent: Option<Rc<State<'a>>>,
    variations: Variations,
}

struct StateIter<'a> {
    node: Option<&'a State<'a>>,
}

struct Searcher<'a> {
    // what we have
    edges_in: &'a HashMap<Int, HashSet<Arc<Edge>>>,
    edges_out: &'a HashMap<Int, HashSet<Arc<Edge>>>,

    // what we want to find
    hash_in: Int,
    hash_out: Int,
    hash_var_in: &'a Variations,
    hash_var_out: &'a Variations,

    // our search queue
    queue_in: BinaryHeap<Reverse<Rc<State<'a>>>>,
    queue_out: BinaryHeap<Reverse<Rc<State<'a>>>>,

    // track where we have been (using u64 hash to skip tranferring ownership)
    visited_in: HashMap<&'a Arc<Edge>, HashMap<u64, Rc<State<'a>>>>,
    visited_out: HashMap<&'a Arc<Edge>, HashMap<u64, Rc<State<'a>>>>,

    // If we need to skip any edges in our search.
    skip_edges: &'a BTreeSet<&'a Arc<Edge>>,
}

// Our graph!
pub struct Graph {
    edges_in: HashMap<Int, HashSet<Arc<Edge>>>,
    edges_out: HashMap<Int, HashSet<Arc<Edge>>>,
}

impl<'a> State<'a> {
    fn new(
        cost: Int,
        edge: &'a Arc<Edge>,
        parent: Option<Rc<State<'a>>>,
        variations: Variations,
    ) -> Self {
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
            variations,
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
        hash_var_in: &'a Variations,
        hash_out: Int,
        hash_var_out: &'a Variations,
        edges_in: &'a HashMap<Int, HashSet<Arc<Edge>>>,
        edges_out: &'a HashMap<Int, HashSet<Arc<Edge>>>,
        skip_edges: &'a BTreeSet<&'a Arc<Edge>>,
    ) -> Self {
        Searcher {
            edges_in,
            edges_out,
            hash_in,
            hash_out,
            hash_var_in,
            hash_var_out,
            queue_in: BinaryHeap::new(),
            queue_out: BinaryHeap::new(),
            visited_in: HashMap::new(),
            visited_out: HashMap::new(),
            skip_edges,
        }
    }

    /// Look for the cheapest path between converters (edges)
    /// A chain of types must match. eg A>B  B>C C>D
    /// Variations are like dependencies on input. They are required
    /// to satisfy that edges traversal. If an edge does not satisfy
    /// the right variations, it can be visited again later when it
    /// the current state has a different set of variations.
    fn search(&mut self) -> Option<Vec<Arc<Edge>>> {
        self.set_queue_in();
        self.set_queue_out();

        // Loop till we run out of options.
        // Search forward and back at the same time.
        // Favour the direction with the least number of options.
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

    fn search_forward(&mut self) -> Option<Vec<Arc<Edge>>> {
        // next state
        let state = match self.queue_in.pop() {
            Some(Reverse(s)) => s,
            _ => return None,
        };

        if self.skip_edges.contains(&state.edge) {
            return None;
        }

        // Check if we have reached our goal and variations are all met
        if state.edge.hash_out == self.hash_out && state.variations.is_superset(self.hash_var_out) {
            return Some(
                state
                    .iter()
                    .map(|s| Arc::clone(&s.edge))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect(),
            );
        }

        // Check if our path intersects the forward search
        if let Some(opposite_states) = self.visited_out.get(&state.edge) {
            for opposite_state in opposite_states.values() {
                // Dependency check
                if !opposite_state.variations.is_subset(match &state.parent {
                    Some(parent) => &parent.variations,
                    None => self.hash_var_in,
                }) {
                    continue;
                }
                return Some(
                    state
                        .iter()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .chain(opposite_state.iter())
                        .map(|s| Arc::clone(&s.edge))
                        .collect(),
                );
            }
        }

        // Mark where we have been
        let edge_entry = self.visited_in.entry(state.edge).or_insert(HashMap::new());
        edge_entry.insert(
            match &state.parent {
                Some(parent) => hash(&parent.variations),
                None => hash(&self.hash_var_in),
            },
            Rc::clone(&state),
        );
        self.add_queue_in(state);
        None
    }

    fn search_backward(&mut self) -> Option<Vec<Arc<Edge>>> {
        let state = match self.queue_out.pop() {
            Some(Reverse(s)) => s,
            _ => return None,
        };

        if self.skip_edges.contains(&state.edge) {
            return None;
        }

        // Check if we have reached our goal and variations dependencies are met
        if state.edge.hash_in == self.hash_in && state.variations.is_subset(self.hash_var_in) {
            return Some(state.iter().map(|s| Arc::clone(&s.edge)).collect());
        }

        // Check if our path intersects the forward search
        if let Some(opposite_states) = self.visited_in.get(&state.edge) {
            for opposite_state in opposite_states.values() {
                // Dependency check
                if !state.variations.is_subset(match &opposite_state.parent {
                    Some(parent) => &parent.variations,
                    None => self.hash_var_in,
                }) {
                    continue;
                }

                return Some(
                    opposite_state
                        .iter()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .chain(state.iter())
                        .map(|s| Arc::clone(&s.edge))
                        .collect(),
                );
            }
        }

        // Mark where we have been
        let edge_entry = self.visited_out.entry(state.edge).or_insert(HashMap::new());
        edge_entry.insert(
            match &state.parent {
                Some(parent) => hash(&parent.variations),
                None => hash(&self.hash_var_out),
            },
            Rc::clone(&state),
        );

        self.add_queue_out(state);
        None
    }

    fn set_queue_in(&mut self) {
        if let Some(edges) = self.edges_in.get(&self.hash_in) {
            for edge in edges {
                // Variation requirement check
                if !edge.hash_var_in.is_subset(self.hash_var_in) {
                    continue;
                }
                self.queue_in.push(Reverse(Rc::new(State::new(
                    edge.cost / (edge.hash_var_in.len() + 1) as Int,
                    &edge,
                    None,
                    // chain method faster?
                    //self.hash_var_in.difference(&edge.hash_var_in).chain(edge.hash_var_out.iter()).cloned().collect(),
                    &(self.hash_var_in - &edge.hash_var_in) | &edge.hash_var_out,
                ))))
            }
        }
    }

    fn set_queue_out(&mut self) {
        if let Some(edges) = self.edges_out.get(&self.hash_out) {
            for edge in edges {
                self.queue_out.push(Reverse(Rc::new(State::new(
                    edge.cost
                        / (self.hash_var_out.intersection(&edge.hash_var_out).count() + 1) as Int,
                    &edge,
                    None,
                    &(self.hash_var_out - &edge.hash_var_out) | &edge.hash_var_in,
                ))))
            }
        }
    }

    fn add_queue_in(&mut self, state: Rc<State<'a>>) {
        if let Some(edges) = self.edges_in.get(&state.edge.hash_out) {
            for edge in edges {
                if self
                    .visited_in
                    .get(&edge)
                    .map_or(false, |v| v.contains_key(&hash(&state.variations)))
                {
                    continue;
                }
                // Variation dependency check
                // Any node visited walking forward needs all its
                // variations to be provided. By the current node and
                // all nodes that came before (unless they already
                // consumed some)
                if !edge.hash_var_in.is_subset(&state.variations) {
                    continue;
                }
                self.queue_in.push(Reverse(Rc::new(State::new(
                    state.cost + edge.cost / (edge.hash_var_in.len() + 1) as Int,
                    &edge,
                    Some(Rc::clone(&state)),
                    &(&state.variations - &edge.hash_var_in) | &edge.hash_var_out,
                ))));
            }
        }
    }

    fn add_queue_out(&mut self, state: Rc<State<'a>>) {
        // Search further into the graph!
        if let Some(edges) = self.edges_out.get(&state.edge.hash_in) {
            for edge in edges {
                if self
                    .visited_out
                    .get(&edge)
                    .map_or(false, |v| v.contains_key(&hash(&state.variations)))
                {
                    continue;
                }
                // Check dependencies
                //if !edge.hash_var_out.is_subset(&state.variations) {
                // TODO: check if this dependency check is needed.
                // The output could potentially provide more than we need.
                // It also doesn't have to produce all the variations
                // we need by itself. More could come from nodes we haven't
                // visited yet.
                //    continue;
                //}
                self.queue_out.push(Reverse(Rc::new(State::new(
                    state.cost
                        + edge.cost
                            / (edge.hash_var_out.intersection(&state.variations).count() + 1)
                                as Int,
                    &edge,
                    Some(Rc::clone(&state)),
                    &(&state.variations - &edge.hash_var_out) | &edge.hash_var_in,
                ))));
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
    pub fn add_edge(
        &mut self,
        cost: Int,
        hash_in: Int,
        hash_var_in: Variations,
        hash_out: Int,
        hash_var_out: Variations,
        hash_func: Int,
    ) {
        let edge_arc = Arc::new(Edge {
            cost,
            hash_in,
            hash_out,
            hash_func,
            hash_var_in,
            hash_var_out,
        });
        let edges_in = self.edges_in.entry(hash_in).or_insert(HashSet::new());
        let edges_out = self.edges_out.entry(hash_out).or_insert(HashSet::new());
        edges_in.insert(Arc::clone(&edge_arc));
        edges_out.insert(edge_arc);
    }

    // Search the graph to find what we want to find
    pub fn search(
        &self,
        hash_in: Int,
        hash_var_in: &Variations,
        hash_out: Int,
        hash_var_out: &Variations,
        skip_edges: &BTreeSet<&Arc<Edge>>,
    ) -> Option<Vec<Arc<Edge>>> {
        let mut searcher = Searcher::new(
            hash_in,
            hash_var_in,
            hash_out,
            hash_var_out,
            &self.edges_in,
            &self.edges_out,
            skip_edges,
        );
        searcher.search()
    }
}

fn hash<H>(hashable: H) -> u64
where
    H: Hash,
{
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    hasher.finish()
}
