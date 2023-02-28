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
mod lookup;

#[cfg(test)]
mod tests;

// INTERNALS
// ================================================================================================

use error::{DataError, KeyError, NodeError};
use hash_db::HashDBRef;
use key::Key;
use node::{ChildSelector, Node, NodeHash};
use storage::NodeStorage;
use tree::null_nodes;

use hashbrown::{HashMap, HashSet};

// RE-EXPORTS
// ================================================================================================

pub use error::TreeError;
pub use hash_db::Hasher;
pub use indexdb::IndexTreeDB;
pub use indexdbmut::IndexTreeDBMut;
pub use proof::StorageProof;
pub use recorder::Recorder;
pub use tree::{IndexTree, IndexTreeMut, KeyedTree, KeyedTreeMut, TreeRecorder};
pub use treedb::{TreeDB, TreeDBBuilder};
pub use treedbmut::{TreeDBMut, TreeDBMutBuilder};

// TYPES
// ================================================================================================

/// The type of value stored in the database backend.
pub type DBValue = Vec<u8>;
