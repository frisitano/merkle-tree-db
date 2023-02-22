use super::{DBValue, Hasher, Key, Node, TreeError};

// INTERFACES
// ================================================================================================

/// A key-value datastore implemented as a database-backed sparse merkle tree.  The tree root,
/// internal node hashes and leaves are all of type H::Out (the hash digest).  The values are of type
/// `Vec<u8>`.  Keys `D` bits long, where `D` is the depth of the tree.   
pub trait Tree<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &Key<N>) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &Key<N>) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn proof(&self, key: &Key<N>) -> Result<Option<Vec<Node<H>>>, TreeError>;
}

pub trait TreeMut<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&mut self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &Key<N>) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &Key<N>) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.
    fn proof(&self, key: &Key<N>) -> Result<Option<Vec<Node<H>>>, TreeError>;

    /// Inserts a value at the provided key.
    fn insert(&mut self, key: &Key<N>, value: DBValue) -> Result<Option<DBValue>, TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, key: &Key<N>) -> Result<Option<DBValue>, TreeError>;
}
