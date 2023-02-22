// #![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod key;
mod node;
mod storage;
mod tree;
mod treedb;
mod treedbmut;

use hashbrown::HashMap;

pub use error::TreeError;
pub use hash_db::Hasher;
pub use key::Key;
pub use node::{ChildSelector, Node, NodeHash};
pub use storage::NodeStorage;
pub use tree::{Tree, TreeMut};
pub use treedb::{TreeDB, TreeDBBuilder};
pub use treedbmut::{TreeDBMut, TreeDBMutBuilder};

#[cfg(test)]
mod tests;

// TYPES
// ================================================================================================

/// The type of value stored in the database backend.
pub type DBValue = Vec<u8>;
