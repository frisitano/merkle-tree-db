use super::{HashMap, Hasher, Node};

// NodeStorage
// ================================================================================================

/// NodeStorage used to store in memory nodes
#[derive(Debug)]
pub struct NodeStorage<H: Hasher> {
    nodes: HashMap<H::Out, (Node<H>, usize)>,
}

impl<H: Hasher> NodeStorage<H> {
    /// create a new empty storage
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// get a node from the storage
    pub fn get(&self, hash: &H::Out) -> Option<&Node<H>> {
        self.nodes.get(hash).map(|(node, _)| node)
    }

    /// insert a node into the storage
    pub fn insert(&mut self, node: Node<H>) {
        let hash = node.hash();
        self.nodes
            .entry(*hash)
            .and_modify(|(node, count)| {
                *node = node.clone();
                *count += 1;
            })
            .or_insert((node, 1));
    }

    /// remove a node from the storage
    pub fn remove(&mut self, hash: &H::Out) -> Option<Node<H>> {
        self.nodes
            .get_mut(hash)
            .and_then(|(node, count)| {
                *count -= 1;
                if *count == 0 {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .and_then(|node| self.nodes.remove(hash).map(|_| node))
    }

    /// iterate over key - value pairs of the storage
    pub fn iter(&self) -> hashbrown::hash_map::Iter<H::Out, (Node<H>, usize)> {
        self.nodes.iter()
    }

    /// drain the storage
    pub fn drain(&mut self) -> hashbrown::hash_map::Drain<H::Out, (Node<H>, usize)> {
        self.nodes.drain()
    }

    /// consume the `NodeStorage` and returns the inner `HashMap`
    pub fn inner(self) -> HashMap<H::Out, (Node<H>, usize)> {
        self.nodes
    }
}
