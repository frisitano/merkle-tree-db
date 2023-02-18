#![cfg_attr(not(feature = "std"), no_std)]

mod key;
mod node;
mod treedb;

pub use key::Key;
pub use node::{ChildSelector, Node, NodeHash};
pub use treedb::TreeDB;

pub use hash_db::Hasher;

#[cfg(test)]
mod tests;

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
/// internal nodes and leaves are all of type H::Out (the hash digest).  The values are of type
/// `Vec<u8>`.  Keys `d` bits long, where `d` is the depth of the tree.   
pub trait Tree<H: Hasher, const N: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth() -> usize {
        N * 8
    }

    /// Returns the value at the provided key.
    fn get_value(&self, key: &Key<N>) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn get_leaf(&self, key: &Key<N>) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn get_proof(&self, key: &Key<N>) -> Result<Option<Vec<(usize, DBValue)>>, TreeError>;
}
