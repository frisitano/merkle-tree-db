// #![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod indexdb;
mod indexdbmut;
mod key;
mod node;
mod proof;
mod recorder;
mod storage;
mod tree;
mod treedb;
mod treedbmut;

use hashbrown::{HashMap, HashSet};

pub use error::{DataError, KeyError, NodeError, TreeError};
pub use hash_db::Hasher;
pub use key::Key;
pub use node::{ChildSelector, Node, NodeHash};
pub use proof::StorageProof;
pub use recorder::Recorder;
pub use storage::NodeStorage;
pub use tree::{null_nodes, IndexTree, IndexTreeMut, KeyedTree, KeyedTreeMut, TreeRecorder};
pub use treedb::{TreeDB, TreeDBBuilder};
pub use treedbmut::{TreeDBMut, TreeDBMutBuilder};

#[cfg(test)]
mod tests;

// TYPES
// ================================================================================================

/// The type of value stored in the database backend.
pub type DBValue = Vec<u8>;
