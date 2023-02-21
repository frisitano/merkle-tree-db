// #![cfg_attr(not(feature = "std"), no_std)]

mod key;
mod node;
mod treedb;
mod treedbmut;

pub use hash_db::Hasher;
pub use key::Key;
pub use node::{ChildSelector, Node, NodeHash};
pub use treedb::{TreeDB, TreeDBBuilder};
pub use treedbmut::{TreeDBMut, TreeDBMutBuilder};

#[cfg(test)]
mod tests;
use treedbmut::null_hashes;

// CONSTANTS
// ================================================================================================

/// The type of value stored in the database backend.
pub type DBValue = Vec<u8>;

// DATA STRUCTURES
// ================================================================================================

/// Errors for the tree library
#[derive(Debug)]
pub enum TreeError {
    DataNotFound,
    DecodeNodeFailed,
    DecodeHashFailed,
    InvalidHeight,
    InconsistentDefaultHashes,
    MissingChild,
    UnexpectedNodeType,
}

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
    fn insert(&mut self, key: &Key<N>, value: DBValue) -> Result<(), TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, key: &Key<N>) -> Result<(), TreeError>;
}
