//! merkle-tree-db is a highly flexible library for working with merkle trees. It supports persistance
//! over any key-value database backend. The merkle tree data structures are stored as sparse merkle
//! trees allowing for efficient lookups, updates and persistence. The library is generic over the
//! hasher used and the depth of the tree. Sparse merkle trees that leverage circuit friendly hash
//! functions (e.g Poseidon, Rescue-Prime) are performant in a ZKP setting and as such this library can
//! serve this purpose. This library supports both indexed merkle trees (max depth 64) and keyed
//! (addressable) merkle trees (max depth usize::MAX).

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

#[cfg(test)]
mod tests;

// INTERNALS
// ================================================================================================

use error::{DataError, KeyError, NodeError};
use key::Key;
use node::{ChildSelector, Node, NodeHash};
use storage::NodeStorage;
use tree::null_nodes;

use hashbrown::{HashMap, HashSet};

// RE-EXPORTS
// ================================================================================================

pub use error::TreeError;
pub use indexdb::{IndexTreeDB, IndexTreeDBBuilder};
pub use indexdbmut::{IndexTreeDBMut, IndexTreeDBMutBuilder};
pub use proof::StorageProof;
pub use recorder::Recorder;
pub use tree::{IndexTree, IndexTreeMut, KeyedTree, KeyedTreeMut, TreeRecorder};
pub use treedb::{TreeDB, TreeDBBuilder};
pub use treedbmut::{TreeDBMut, TreeDBMutBuilder};

pub use hash_db::{HashDB, HashDBRef, Hasher};

// TYPES
// ================================================================================================

/// The type of value stored in the database backend.
pub type DBValue = Vec<u8>;
