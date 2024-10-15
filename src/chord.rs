use rand::Rng;
use std::collections::BTreeMap;

pub const MAX_ID: u16 = 64; // Keyspace size (mod MAX_ID)

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u16,
}

pub struct Chord {
    // Sorted map of nodes in the Chord ring
    nodes: BTreeMap<u16, Node>,
}

impl Chord {
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
        }
    }

    /// Add a node at a random position in the Chord ring
    pub fn add_node(&mut self) -> Result<u16, ChordError> {
        let mut rng = rand::thread_rng();
        if self.nodes.len() == MAX_ID.into() {
            Err(ChordError::RingIsFull)
        }
        else {
            let mut new_id = rng.gen_range(0..MAX_ID);

            while self.nodes.contains_key(&new_id) {
                new_id = rng.gen_range(0..MAX_ID); // Pick another random ID within the keyspace
            }
    
            let node = Node { id: new_id };
            self.nodes.insert(new_id, node);
            Ok(new_id)
        }
    }

    /// Delete a node by its id
    pub fn delete_node(&mut self, id: u16) -> Result<(), ChordError> {
        if self.nodes.remove(&id).is_some() {
            Ok(())
        } else {
            Err(ChordError::NodeDoesNotExist)
        }
    }

    /// Find the node responsible for a given key
    pub fn search(&self, key: u16) -> Result<&Node, ChordError> {
        if self.nodes.is_empty() {
            return Err(ChordError::NoNodesExist);
        }

        if key >= MAX_ID {
            return Err(ChordError::OutOfRange);
        }

        // Get the node responsible for the key (the smallest node ID >= key, or the first node in the ring)
        match self.nodes.range(key..).next() {
            Some((_, node)) => {
                Ok(node)
            }
            None => {
                // If no node ID is >= key, wrap around to the first node
                let first_node = self.nodes.iter().next().unwrap();
                Ok(first_node.1)
            }
        }
    }

    /// Print the current Chord ring for visualization
    pub fn get_ring(&self) -> Vec<&u16> {
        self.nodes.keys().collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub enum ChordError {
    NoNodesExist,
    NodeDoesNotExist,
    RingIsFull,
    OutOfRange,
}