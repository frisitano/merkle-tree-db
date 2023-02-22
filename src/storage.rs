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

    /// check if a node is in the storage
    pub fn contains(&self, hash: &H::Out) -> bool {
        self.nodes.contains_key(hash)
    }

    /// check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// iterate over the storage
    pub fn iter(&self) -> hashbrown::hash_map::Iter<H::Out, (Node<H>, usize)> {
        self.nodes.iter()
    }

    /// turns the storage into an iterator
    pub fn into_iter(self) -> hashbrown::hash_map::IntoIter<H::Out, (Node<H>, usize)> {
        self.nodes.into_iter()
    }

    /// exposes the inner storage
    pub fn inner(self) -> HashMap<H::Out, (Node<H>, usize)> {
        self.nodes
    }
}
