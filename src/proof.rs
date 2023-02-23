use super::{HashSet, Hasher};
use core::marker::PhantomData;
use hash_db::{AsHashDB, Prefix, EMPTY_PREFIX};
use memory_db::{KeyFunction, MemoryDB};
use std::iter::IntoIterator;

// StorageProof
// ================================================================================================

/// A proof that some set of key-value pairs are included in a sparse merkle tree.
pub struct StorageProof {
    nodes: HashSet<Vec<u8>>,
}

impl StorageProof {
    /// Creates a new storage proof from the provided set of nodes.
    pub fn new(nodes: impl IntoIterator<Item = Vec<u8>>) -> Self {
        Self {
            nodes: HashSet::from_iter(nodes),
        }
    }

    /// Returns an empty storage proof.
    pub fn empty() -> Self {
        Self {
            nodes: HashSet::new(),
        }
    }

    /// Returns whether this proof is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Consumes the storage proof and returns the set of nodes.
    pub fn into_nodes(self) -> HashSet<Vec<u8>> {
        self.nodes
    }

    /// Consumes the storage proof and returns a memory db containing the nodes.
    pub fn into_memory_db<H: Hasher>(self) -> MemoryDB<H, NoopKey<H>, Vec<u8>> {
        self.into()
    }
}

// MemoryDB
// ================================================================================================

pub struct NoopKey<H: Hasher>(PhantomData<H>);

impl<H: Hasher> KeyFunction<H> for NoopKey<H> {
    type Key = Vec<u8>;

    fn key(hash: &H::Out, _prefix: Prefix) -> Vec<u8> {
        hash.as_ref().to_vec()
    }
}

/// Implement from StorageProof for MemoryDB
impl<H: Hasher> From<StorageProof> for MemoryDB<H, NoopKey<H>, Vec<u8>> {
    fn from(proof: StorageProof) -> Self {
        let mut db = MemoryDB::<H, NoopKey<H>, Vec<u8>>::default();
        for node in proof.into_nodes().into_iter() {
            db.as_hash_db_mut()
                .emplace(H::hash(&node[1..]), EMPTY_PREFIX, node);
        }
        db
    }
}
