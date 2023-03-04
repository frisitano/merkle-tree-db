use super::{HashMap, Hasher, Node, StorageProof, TreeRecorder};

// Recorder
// ================================================================================================

/// Recorder to record database reads.
pub struct Recorder<H: Hasher> {
    nodes: HashMap<H::Out, Node<H>>,
}

/// Implement default for Recorder.
impl<H: Hasher> Default for Recorder<H> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of Recorder.
impl<H: Hasher> Recorder<H> {
    /// Creates a new empty recorder.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::default(),
        }
    }

    /// Drain the recorder and return the recorded nodes.
    pub fn drain(&mut self) -> hashbrown::hash_map::Drain<H::Out, Node<H>> {
        self.nodes.drain()
    }

    /// Drain the recorder and return the recorded nodes as a storage proof.
    pub fn drain_storage_proof(self) -> StorageProof {
        StorageProof::new(self.nodes.into_iter().map(|(_, node)| node.into()))
    }

    /// Returns the recorded nodes as a storage proof.
    pub fn to_storage_proof(&self) -> StorageProof {
        StorageProof::new(self.nodes.values().cloned().map(|node| node.into()))
    }
}

/// Implementation of TreeRecorder for Recorder.
impl<H: Hasher> TreeRecorder<H> for Recorder<H> {
    fn record(&mut self, node: &Node<H>) {
        self.nodes.insert(*node.hash(), node.clone());
    }
}
