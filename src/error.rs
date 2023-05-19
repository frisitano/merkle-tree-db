// TREE ERROR
// ================================================================================================

/// Errors associated with the tree. These errors are returned by the tree methods and wrap
/// errors returned by the underlying components, these include:
/// - DataError - errors associated with the underlying data the tree is built on
/// - NodeError - errors associated with the nodes in the tree
/// - DepthTooLarge - error returned when the specified tree depth is too large
/// - KeyError - error associated with the key used to access the tree
use super::rstd::{string::String, vec::Vec};

#[derive(Debug, PartialEq, Eq)]
pub enum TreeError {
    DataError(DataError),
    NodeError(NodeError),
    DepthTooLarge(usize, usize),
    KeyError(KeyError),
}

impl core::fmt::Display for TreeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TreeError::*;
        match self {
            DataError(err) => write!(f, "Data Error: {err}"),
            NodeError(err) => write!(f, "Node Error: {err}"),
            DepthTooLarge(actual, max) => {
                write!(f, "depth {actual} too large - max supported depth is {max}",)
            }
            KeyError(err) => write!(f, "key error: {err}"),
        }
    }
}

// DATA ERROR
// ================================================================================================

/// Errors associated with the underlying data the tree is built on.
#[derive(Debug, PartialEq, Eq)]
pub enum DataError {
    DatabaseDataNotFound(Vec<u8>),
    NullNodeDataNotFound(Vec<u8>),
    InMemoryDataNotFound(Vec<u8>),
    InMemoryNotSupported,
}

impl core::fmt::Display for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use DataError::*;
        match self {
            DatabaseDataNotFound(hash) => {
                write!(f, "database data not found for hash {hash:?}")
            }
            NullNodeDataNotFound(hash) => {
                write!(f, "null node data not found for hash {hash:?}")
            }
            InMemoryNotSupported => write!(f, "in-memory data not supported for immutable tree"),
            InMemoryDataNotFound(hash) => {
                write!(f, "in-memory data not found for hash {hash:?}")
            }
        }
    }
}

// NODE ERROR
// ================================================================================================

/// Errors associated with the nodes in the tree.
#[derive(Debug, PartialEq, Eq)]
pub enum NodeError {
    DecodeNodeEmptyValue,
    DecodeNodeNoData,
    DecodeNodeInvalidPrefix(u8),
    DecodeNodeHashFailed(Vec<u8>),
    DecodeNodeInvalidLength(usize, usize),
    InconsistentDefaultHashes,
    InvalidNodeType(String, String),
}

impl core::fmt::Display for NodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use NodeError::*;
        match self {
            DecodeNodeEmptyValue => write!(f, "decode node failed - empty value provided"),
            DecodeNodeNoData => write!(f, "decode node failed - no data provided"),
            DecodeNodeInvalidPrefix(prefix) => {
                write!(f, "decode node failed - invalid prefix {prefix}",)
            }
            DecodeNodeHashFailed(data) => {
                write!(
                    f,
                    "decode node failed - hash decode failed for data {data:?}",
                )
            }
            DecodeNodeInvalidLength(expected, actual) => {
                write!(
                    f,
                    "decode node failed - invalid length - expected {expected}, got {actual}",
                )
            }
            InconsistentDefaultHashes => {
                write!(f, "inconsistent default hashes")
            }
            InvalidNodeType(expected, actual) => {
                write!(
                    f,
                    "invalid node type - method not supported - expected {expected}, got {actual}",
                )
            }
        }
    }
}

// KEY ERROR
// ================================================================================================

/// Errors associated with the keys used to access the tree.
#[derive(Debug, PartialEq, Eq)]
pub enum KeyError {
    IncorrectKeySize(usize, usize),
    BitIndexOutOfBounds(usize, usize),
    LeafIndexOutOfBounds(u64, u64),
}

impl core::fmt::Display for KeyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use KeyError::*;
        match self {
            IncorrectKeySize(expected, actual) => {
                write!(f, "incorrect key size - expected {expected}, got {actual}",)
            }
            BitIndexOutOfBounds(bit, max) => {
                write!(
                    f,
                    "bit index out of bounds - index {bit} is out of range - max {max}",
                )
            }
            LeafIndexOutOfBounds(index, max) => {
                write!(
                    f,
                    "leaf index out of bounds - index {index} is out of range - max {max}",
                )
            }
        }
    }
}
