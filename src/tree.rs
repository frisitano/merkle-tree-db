use super::{DBValue, HashMap, Hasher, Node, NodeHash, TreeError};

// TRAITS
// ================================================================================================

pub trait KeyedTree<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn proof(&self, key: &[u8]) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        key: &[u8],
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

/// A mutable key-value datastore implemented as a database-backed sparse merkle tree.
pub trait KeyedTreeMut<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&mut self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.
    fn proof(&self, key: &[u8]) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Inserts a value at the provided key.
    fn insert(&mut self, key: &[u8], value: DBValue) -> Result<Option<DBValue>, TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        key: &[u8],
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

pub trait IndexTree<H: Hasher, const D: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        D * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, index: &u64) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn proof(&self, index: &u64) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

pub trait IndexTreeMut<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&mut self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, index: &u64) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.
    fn proof(&self, index: &u64) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Inserts a value at the provided key.
    fn insert(&mut self, index: &u64, value: DBValue) -> Result<Option<DBValue>, TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

/// A trait that allows recording of tree nodes.
pub trait TreeRecorder<H: Hasher> {
    fn record(&mut self, node: &Node<H>);
}

// Helpers
// ================================================================================================

/// Return the HashMap hashing node hash to Node for null nodes of a tree of depth D
pub fn null_nodes<H: Hasher>(depth: usize) -> (HashMap<H::Out, Node<H>>, H::Out) {
    let mut hashes = HashMap::with_capacity(depth);
    let mut current_hash = H::hash(&[]);

    hashes.insert(
        current_hash,
        Node::Value {
            hash: current_hash,
            value: vec![],
        },
    );

    for _ in 0..depth {
        let next_hash = H::hash(&[current_hash.as_ref(), current_hash.as_ref()].concat());
        hashes.insert(
            next_hash,
            Node::Inner {
                hash: next_hash,
                left: NodeHash::Default(current_hash),
                right: NodeHash::Default(current_hash),
            },
        );
        current_hash = next_hash;
    }

    (hashes, current_hash)
}
